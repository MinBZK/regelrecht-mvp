"""Protocols and data structures for the splitting system."""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import TYPE_CHECKING, Protocol

from harvester.models import Article, Reference

if TYPE_CHECKING:
    from lxml import etree


@dataclass
class ElementSpec:
    """Declarative specification of an element in the hierarchy.

    Defines the structural relationships and behavior for an XML element
    in the Dutch law hierarchy.
    """

    tag: str
    """XML tag name (without namespace)."""

    children: list[str] = field(default_factory=list)
    """Valid child element tags that contribute to structure.

    Listed in priority order - first match wins when walking the tree.
    """

    number_source: str | None = None
    """XPath to the child element that provides numbering.

    Examples: "lidnr" for lid, "li.nr" for li, "kop/nr" for artikel.
    """

    content_tags: list[str] = field(default_factory=list)
    """Child tags that contain text content (e.g., ["al"])."""

    is_split_point: bool = False
    """Whether this element can be a split boundary.

    When True, this element may produce an ArticleComponent.
    """

    skip_for_number: list[str] = field(default_factory=list)
    """Child tags to skip when extracting content (e.g., ["lidnr", "li.nr"])."""


@dataclass
class SplitContext:
    """Context for splitting operations.

    Carries state through the recursive tree walk.
    """

    bwb_id: str
    """BWB identifier for the law being processed."""

    date: str
    """Effective date in YYYY-MM-DD format."""

    base_url: str
    """Base URL for the current article."""

    number_parts: list[str] = field(default_factory=list)
    """Accumulated number parts for dot notation (e.g., ["1", "1", "a"])."""

    depth: int = 0
    """Current depth in the hierarchy (0 = artikel level)."""

    max_depth: int | None = None
    """Optional maximum depth to split to."""

    def with_number(self, number: str) -> SplitContext:
        """Create a new context with an additional number part."""
        return SplitContext(
            bwb_id=self.bwb_id,
            date=self.date,
            base_url=self.base_url,
            number_parts=[*self.number_parts, number],
            depth=self.depth + 1,
            max_depth=self.max_depth,
        )


class SplitStrategy(Protocol):
    """Protocol for configurable splitting strategies.

    Implementations determine where to split and how to extract numbers.
    """

    def should_split_here(
        self,
        elem: etree._Element,
        spec: ElementSpec,
        context: SplitContext,
    ) -> bool:
        """Determine if this element should produce a component.

        Args:
            elem: The XML element being processed
            spec: The element specification
            context: Current split context

        Returns:
            True if this element should produce an ArticleComponent
        """
        ...

    def get_number(
        self,
        elem: etree._Element,
        spec: ElementSpec,
    ) -> str | None:
        """Extract the number/identifier for this element.

        Args:
            elem: The XML element
            spec: The element specification

        Returns:
            The extracted number, or None if not found
        """
        ...


@dataclass
class ArticleComponent:
    """Represents a lowest-level component of an article."""

    number_parts: list[str]  # e.g., ["1", "1", "a"] for artikel 1, lid 1, onderdeel a
    text: str
    base_url: str  # Base URL for the article (without fragment)
    references: list[Reference] = field(default_factory=list)

    def to_number(self) -> str:
        """Convert number parts to dot notation."""
        return ".".join(self.number_parts)

    def to_article(self) -> Article:
        """Convert to Article object with reference definitions appended to text."""
        text = self.text

        # Append reference definitions if there are any references
        if self.references:
            ref_lines = []
            for ref in self.references:
                url = ref.to_wetten_url()
                ref_lines.append(f"[{ref.id}]: {url}")
            text = f"{text}\n\n" + "\n".join(ref_lines)

        return Article(
            number=self.to_number(),
            text=text,
            url=self.base_url,
            references=self.references,
        )

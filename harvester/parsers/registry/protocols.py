"""Protocol definitions for the element registry system."""

from __future__ import annotations

from dataclasses import dataclass, field
from enum import Enum, auto
from typing import TYPE_CHECKING, Callable, Protocol

if TYPE_CHECKING:
    from lxml import etree

    from harvester.parsers.content_parser import ReferenceCollector


class ElementType(Enum):
    """Classification of element types for processing strategy."""

    STRUCTURAL = auto()  # Container elements (artikel, lid, lijst)
    INLINE = auto()  # Text-level elements (extref, nadruk)
    SKIP = auto()  # Elements to ignore completely


@dataclass
class ParseResult:
    """Result from parsing an element."""

    text: str
    """The extracted text content."""


@dataclass
class ParseContext:
    """Context passed through parsing operations."""

    collector: ReferenceCollector | None = None
    """Collector for reference-style links."""

    bwb_id: str = ""
    """BWB identifier for the current law."""

    date: str = ""
    """Effective date in YYYY-MM-DD format."""

    number_parts: list[str] = field(default_factory=list)
    """Current article number parts for building dot notation."""

    base_url: str = ""
    """Base URL for the current article."""


# Type alias for recursive processing function
RecurseFn = Callable[["etree._Element", ParseContext], ParseResult]


class ElementHandler(Protocol):
    """Protocol for element handlers.

    Handlers are responsible for processing a specific type of XML element
    and returning its text content. They receive a `recurse` function to
    process child elements.
    """

    @property
    def element_type(self) -> ElementType:
        """Return the type classification of this element."""
        ...

    def can_handle(self, elem: etree._Element, context: ParseContext) -> bool:
        """Check if this handler can process the given element.

        Args:
            elem: The XML element to check
            context: Current parsing context

        Returns:
            True if this handler can process the element
        """
        ...

    def handle(
        self,
        elem: etree._Element,
        context: ParseContext,
        recurse: RecurseFn,
    ) -> ParseResult:
        """Process the element and return parsed text.

        Args:
            elem: The XML element to process
            context: Current parsing context
            recurse: Function to call for recursive child processing

        Returns:
            ParseResult containing the extracted text
        """
        ...

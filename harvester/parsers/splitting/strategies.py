"""Split strategy implementations."""

from __future__ import annotations

from dataclasses import dataclass
from typing import TYPE_CHECKING

from harvester.parsers.splitting.protocols import ElementSpec, SplitContext

if TYPE_CHECKING:
    from lxml import etree


@dataclass
class LeafSplitStrategy:
    """Default strategy: split at every leaf (deepest) level.

    This matches the current behavior of article_splitter.py:
    - Split at artikel, lid, and li elements
    - Extract intro text before structural children
    - Recurse into nested structures
    """

    def should_split_here(
        self,
        elem: etree._Element,
        spec: ElementSpec,
        context: SplitContext,
    ) -> bool:
        """Split at any element marked as a split point."""
        return spec.is_split_point

    def get_number(
        self,
        elem: etree._Element,
        spec: ElementSpec,
    ) -> str | None:
        """Extract number from the configured source.

        Handles special cases:
        - XPath-style paths like "kop/nr"
        - Direct child elements like "lidnr"
        - List items (li.nr) with trailing periods removed
        - Unmarked lists with sequential numbering
        """
        if not spec.number_source:
            return None

        # Handle XPath-style paths
        nr_elem = elem.find(spec.number_source)
        if nr_elem is None or not nr_elem.text:
            # Try fallback for artikel with label attribute
            if spec.tag == "artikel":
                return self._get_artikel_number_fallback(elem)
            return None

        nr = nr_elem.text.strip()

        # Remove trailing period only for li.nr (not lidnr)
        if spec.number_source == "li.nr" and nr.endswith("."):
            nr = nr[:-1]

        return nr if nr else None

    def _get_artikel_number_fallback(self, elem: etree._Element) -> str | None:
        """Get artikel number from label attribute if nr element is missing."""
        label = elem.get("label", "")
        if label.startswith("Artikel "):
            return label.replace("Artikel ", "").strip()
        if label:
            return label
        return None


@dataclass
class DepthLimitedStrategy:
    """Strategy that merges the last N levels.

    Use this to control how deep to split. For example:
    - merge_depth=0: Split at all levels (same as LeafSplitStrategy)
    - merge_depth=1: Merge the deepest level with its parent
    - merge_depth=2: Merge the two deepest levels

    This is useful when you want coarser splitting, e.g., keeping
    onderdelen together with their parent lid.
    """

    merge_depth: int = 1
    """Number of levels from the bottom to merge."""

    _base_strategy: LeafSplitStrategy | None = None

    def __post_init__(self) -> None:
        """Initialize the base strategy for number extraction."""
        self._base_strategy = LeafSplitStrategy()

    def should_split_here(
        self,
        elem: etree._Element,
        spec: ElementSpec,
        context: SplitContext,
    ) -> bool:
        """Split only if we're not in the merge zone.

        The merge zone is the last `merge_depth` levels from max_depth.
        """
        if not spec.is_split_point:
            return False

        if context.max_depth is not None:
            merge_zone_start = context.max_depth - self.merge_depth
            if context.depth >= merge_zone_start:
                return False

        return True

    def get_number(
        self,
        elem: etree._Element,
        spec: ElementSpec,
    ) -> str | None:
        """Delegate to base strategy for number extraction."""
        if self._base_strategy is None:
            self._base_strategy = LeafSplitStrategy()
        return self._base_strategy.get_number(elem, spec)

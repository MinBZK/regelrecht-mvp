"""Split engine that orchestrates article splitting using the hierarchy registry."""

from __future__ import annotations

from typing import TYPE_CHECKING

from harvester.parsers.article_splitter import ArticleComponent
from harvester.parsers.content_parser import ReferenceCollector
from harvester.parsers.registry.registry import get_tag_name
from harvester.parsers.splitting.protocols import (
    ElementSpec,
    SplitContext,
    SplitStrategy,
)
from harvester.parsers.splitting.registry import HierarchyRegistry
from harvester.parsers.text_extractor import extract_inline_text

if TYPE_CHECKING:
    from lxml import etree


class SplitEngine:
    """Engine for splitting articles using hierarchy schema.

    Walks the XML tree according to the hierarchy specification and
    produces ArticleComponent objects at split points.
    """

    def __init__(
        self,
        hierarchy: HierarchyRegistry,
        strategy: SplitStrategy,
    ) -> None:
        """Initialize the engine.

        Args:
            hierarchy: Registry of element specifications
            strategy: Strategy for split decisions and number extraction
        """
        self._hierarchy = hierarchy
        self._strategy = strategy

    def split(
        self,
        elem: etree._Element,
        context: SplitContext,
    ) -> list[ArticleComponent]:
        """Split an element into components based on hierarchy.

        Args:
            elem: The XML element to split
            context: Current split context

        Returns:
            List of ArticleComponent objects
        """
        tag = get_tag_name(elem)
        spec = self._hierarchy.get_spec(tag)

        if spec is None:
            return []

        components: list[ArticleComponent] = []

        # Get number for this element and update context
        number = self._strategy.get_number(elem, spec)
        if number:
            context = context.with_number(number)

        # Find structural children
        structural_children = self._find_structural_children(elem, spec)

        if structural_children:
            # Has structural children - extract intro and recurse
            components.extend(
                self._process_with_structural_children(
                    elem, spec, context, structural_children
                )
            )
        else:
            # Leaf node - extract content if this is a split point
            if self._strategy.should_split_here(elem, spec, context):
                component = self._extract_leaf_content(elem, spec, context)
                if component:
                    components.append(component)

        return components

    def _find_structural_children(
        self,
        elem: etree._Element,
        spec: ElementSpec,
    ) -> list[etree._Element]:
        """Find structural children according to spec.

        Checks children in priority order and returns the first
        matching type found.
        """
        for child_tag in spec.children:
            children = [child for child in elem if get_tag_name(child) == child_tag]
            if children:
                return children
        return []

    def _process_with_structural_children(
        self,
        elem: etree._Element,
        spec: ElementSpec,
        context: SplitContext,
        structural_children: list[etree._Element],
    ) -> list[ArticleComponent]:
        """Process an element that has structural children.

        Extracts intro text before the structural children, then
        recurses into each structural child.
        """
        components: list[ArticleComponent] = []

        # Extract intro text before structural children
        if self._strategy.should_split_here(elem, spec, context):
            intro_component = self._extract_intro_text(
                elem, spec, context, structural_children
            )
            if intro_component:
                components.append(intro_component)

        # Recurse into structural children
        for child in structural_children:
            components.extend(self.split(child, context))

        return components

    def _extract_intro_text(
        self,
        elem: etree._Element,
        spec: ElementSpec,
        context: SplitContext,
        structural_children: list[etree._Element],
    ) -> ArticleComponent | None:
        """Extract intro text that appears before structural children."""
        collector = ReferenceCollector()
        parts: list[str] = []

        # Get the first structural child to know where to stop
        first_structural = structural_children[0] if structural_children else None

        for child in elem:
            # Stop when we hit the first structural child
            if child is first_structural:
                break

            child_tag = get_tag_name(child)

            # Skip number elements
            if child_tag in spec.skip_for_number:
                continue

            # Extract content from content tags
            if child_tag in spec.content_tags:
                text = extract_inline_text(child, collector)
                if text:
                    parts.append(text)

        if not parts:
            return None

        return ArticleComponent(
            number_parts=context.number_parts.copy(),
            text=" ".join(parts).strip(),
            base_url=context.base_url,
            references=collector.references.copy(),
        )

    def _extract_leaf_content(
        self,
        elem: etree._Element,
        spec: ElementSpec,
        context: SplitContext,
    ) -> ArticleComponent | None:
        """Extract all content from a leaf element."""
        collector = ReferenceCollector()
        parts: list[str] = []

        for child in elem:
            child_tag = get_tag_name(child)

            # Skip number elements
            if child_tag in spec.skip_for_number:
                continue

            # Extract content from content tags
            if child_tag in spec.content_tags:
                text = extract_inline_text(child, collector)
                if text:
                    parts.append(text)
            elif not self._hierarchy.is_structural(child_tag):
                # Also extract from non-structural elements
                text = extract_inline_text(child, collector)
                if text:
                    parts.append(text)

        if not parts:
            return None

        return ArticleComponent(
            number_parts=context.number_parts.copy(),
            text=" ".join(parts).strip(),
            base_url=context.base_url,
            references=collector.references.copy(),
        )

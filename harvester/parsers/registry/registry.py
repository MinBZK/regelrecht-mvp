"""Element registry for mapping tag names to handlers."""

from __future__ import annotations

from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from lxml import etree

    from harvester.parsers.registry.protocols import (
        ElementHandler,
        ParseContext,
    )


def get_tag_name(elem: etree._Element) -> str:
    """Get tag name without namespace prefix.

    Args:
        elem: XML element

    Returns:
        Tag name without namespace (e.g., "artikel" not "{ns}artikel")
    """
    tag = elem.tag
    if isinstance(tag, str) and "}" in tag:
        return tag.split("}")[-1]
    return tag if isinstance(tag, str) else ""


class ElementRegistry:
    """Registry mapping element names to handlers.

    The registry allows registering handlers for specific tag names,
    as well as marking tags to be skipped entirely.
    """

    def __init__(self) -> None:
        """Initialize an empty registry."""
        self._handlers: dict[str, ElementHandler] = {}
        self._skip_tags: set[str] = set()

    def register(self, tag_name: str, handler: ElementHandler) -> None:
        """Register a handler for a specific tag name.

        Args:
            tag_name: The XML tag name (without namespace)
            handler: The handler to use for this tag
        """
        self._handlers[tag_name] = handler

    def skip(self, *tag_names: str) -> None:
        """Mark tags as skip (don't process, return empty).

        Args:
            tag_names: Tag names to skip
        """
        self._skip_tags.update(tag_names)

    def get_handler(
        self,
        elem: etree._Element,
        context: ParseContext,
    ) -> ElementHandler | None:
        """Get the appropriate handler for an element.

        Args:
            elem: The XML element to get a handler for
            context: Current parsing context

        Returns:
            The handler if found, None if element should be skipped
        """
        tag_name = get_tag_name(elem)

        if tag_name in self._skip_tags:
            return None

        if tag_name in self._handlers:
            handler = self._handlers[tag_name]
            if handler.can_handle(elem, context):
                return handler

        return None

    def should_skip(self, tag_name: str) -> bool:
        """Check if a tag should be skipped.

        Args:
            tag_name: The tag name to check

        Returns:
            True if the tag should be skipped
        """
        return tag_name in self._skip_tags

    def has_handler(self, tag_name: str) -> bool:
        """Check if a handler is registered for a tag.

        Args:
            tag_name: The tag name to check

        Returns:
            True if a handler is registered
        """
        return tag_name in self._handlers

    def registered_tags(self) -> set[str]:
        """Return set of all registered tag names.

        Returns:
            Set of tag names with registered handlers
        """
        return set(self._handlers.keys())

    def skipped_tags(self) -> set[str]:
        """Return set of all skipped tag names.

        Returns:
            Set of tag names marked as skip
        """
        return self._skip_tags.copy()

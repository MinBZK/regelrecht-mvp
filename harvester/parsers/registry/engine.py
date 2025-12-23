"""Parse engine that orchestrates element parsing using the registry."""

from __future__ import annotations

from typing import TYPE_CHECKING

from harvester.parsers.registry.protocols import ParseContext, ParseResult
from harvester.parsers.registry.registry import ElementRegistry, get_tag_name

if TYPE_CHECKING:
    from lxml import etree


class UnknownElementError(Exception):
    """Raised when encountering an element without a registered handler."""

    def __init__(self, tag_name: str, context: str = "") -> None:
        """Initialize the error.

        Args:
            tag_name: The unhandled tag name
            context: Additional context about where the element was found
        """
        self.tag_name = tag_name
        msg = f"No handler for element <{tag_name}>"
        if context:
            msg = f"{msg} in {context}"
        super().__init__(msg)


class ParseEngine:
    """Engine that orchestrates element parsing using the registry.

    The engine walks the XML tree and dispatches elements to their
    registered handlers. It raises UnknownElementError for any element
    that has no handler and is not marked as skip.
    """

    def __init__(self, registry: ElementRegistry) -> None:
        """Initialize the engine with a registry.

        Args:
            registry: The element registry to use for handler lookup
        """
        self._registry = registry

    def parse(
        self,
        elem: etree._Element | None,
        context: ParseContext,
    ) -> ParseResult:
        """Parse an element tree recursively.

        Args:
            elem: The XML element to parse (or None)
            context: Current parsing context

        Returns:
            ParseResult containing the extracted text

        Raises:
            UnknownElementError: If an element has no handler and is not skipped
        """
        if elem is None:
            return ParseResult(text="")

        tag_name = get_tag_name(elem)

        # Skip marked elements
        if self._registry.should_skip(tag_name):
            return ParseResult(text="")

        # Get handler
        handler = self._registry.get_handler(elem, context)

        if handler is not None:
            # Handler exists - let it process
            def recurse(
                child: etree._Element,
                ctx: ParseContext,
            ) -> ParseResult:
                return self.parse(child, ctx)

            return handler.handle(elem, context, recurse)

        # No handler - raise error
        raise UnknownElementError(tag_name)

    def parse_children(
        self,
        elem: etree._Element,
        context: ParseContext,
        separator: str = "\n\n",
    ) -> ParseResult:
        """Parse all children of an element.

        Utility method for handlers that need to process all children.

        Args:
            elem: The parent element
            context: Current parsing context
            separator: String to join child results with

        Returns:
            ParseResult with joined text from all children
        """
        parts: list[str] = []

        for child in elem:
            result = self.parse(child, context)
            if result.text:
                parts.append(result.text)

        return ParseResult(text=separator.join(parts))

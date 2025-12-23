"""Tests for ParseEngine and UnknownElementError."""

import pytest
from lxml import etree

from harvester.parsers.registry import (
    ElementRegistry,
    ElementType,
    ParseContext,
    ParseEngine,
    ParseResult,
    UnknownElementError,
)


class MockHandler:
    """Mock handler for testing."""

    def __init__(self, text: str = "mock") -> None:
        self._text = text

    @property
    def element_type(self) -> ElementType:
        return ElementType.INLINE

    def can_handle(self, elem: etree._Element, context: ParseContext) -> bool:
        return True

    def handle(self, elem, context, recurse) -> ParseResult:
        return ParseResult(text=self._text)


class RecursiveHandler:
    """Handler that recurses into children."""

    @property
    def element_type(self) -> ElementType:
        return ElementType.STRUCTURAL

    def can_handle(self, elem: etree._Element, context: ParseContext) -> bool:
        return True

    def handle(self, elem, context, recurse) -> ParseResult:
        parts = []
        for child in elem:
            result = recurse(child, context)
            if result.text:
                parts.append(result.text)
        return ParseResult(text=" ".join(parts))


class TestUnknownElementError:
    """Tests for UnknownElementError exception."""

    def test_error_message_basic(self) -> None:
        """Test basic error message."""
        error = UnknownElementError("unknown-tag")
        assert str(error) == "No handler for element <unknown-tag>"
        assert error.tag_name == "unknown-tag"

    def test_error_message_with_context(self) -> None:
        """Test error message with context."""
        error = UnknownElementError("unknown-tag", context="parsing article 5")
        assert str(error) == "No handler for element <unknown-tag> in parsing article 5"
        assert error.tag_name == "unknown-tag"

    def test_error_is_exception(self) -> None:
        """Test that error is a proper exception."""
        error = UnknownElementError("test")
        assert isinstance(error, Exception)

        with pytest.raises(UnknownElementError) as exc_info:
            raise error
        assert exc_info.value.tag_name == "test"


class TestParseEngine:
    """Tests for ParseEngine class."""

    def test_parse_with_registered_handler(self) -> None:
        """Test parsing element with registered handler."""
        registry = ElementRegistry()
        registry.register("test", MockHandler(text="parsed content"))
        engine = ParseEngine(registry)

        elem = etree.fromstring("<test>ignored</test>")
        context = ParseContext()

        result = engine.parse(elem, context)
        assert result.text == "parsed content"

    def test_parse_none_returns_empty(self) -> None:
        """Test parsing None returns empty result."""
        registry = ElementRegistry()
        engine = ParseEngine(registry)

        result = engine.parse(None, ParseContext())
        assert result.text == ""

    def test_parse_skipped_element_returns_empty(self) -> None:
        """Test parsing skipped element returns empty."""
        registry = ElementRegistry()
        registry.skip("meta-data")
        engine = ParseEngine(registry)

        elem = etree.fromstring("<meta-data>content</meta-data>")
        result = engine.parse(elem, ParseContext())
        assert result.text == ""

    def test_parse_unknown_element_raises_error(self) -> None:
        """Test parsing unknown element raises UnknownElementError."""
        registry = ElementRegistry()
        engine = ParseEngine(registry)

        elem = etree.fromstring("<unknown>content</unknown>")

        with pytest.raises(UnknownElementError, match="unknown"):
            engine.parse(elem, ParseContext())

    def test_parse_recursive(self) -> None:
        """Test recursive parsing through handler."""
        registry = ElementRegistry()
        registry.register("parent", RecursiveHandler())
        registry.register("child", MockHandler(text="child-text"))
        engine = ParseEngine(registry)

        elem = etree.fromstring("<parent><child/><child/></parent>")
        result = engine.parse(elem, ParseContext())
        assert result.text == "child-text child-text"

    def test_parse_children_utility(self) -> None:
        """Test parse_children utility method."""
        registry = ElementRegistry()
        registry.register("item", MockHandler(text="item"))
        engine = ParseEngine(registry)

        elem = etree.fromstring("<container><item/><item/><item/></container>")
        result = engine.parse_children(elem, ParseContext(), separator=", ")
        assert result.text == "item, item, item"

    def test_parse_children_with_separator(self) -> None:
        """Test parse_children with custom separator."""
        registry = ElementRegistry()
        registry.register("line", MockHandler(text="line"))
        engine = ParseEngine(registry)

        elem = etree.fromstring("<doc><line/><line/></doc>")
        result = engine.parse_children(elem, ParseContext(), separator="\n")
        assert result.text == "line\nline"

    def test_nested_unknown_element_raises_error(self) -> None:
        """Test that unknown element in nested structure raises error."""
        registry = ElementRegistry()
        registry.register("parent", RecursiveHandler())
        # Note: "unknown" is not registered
        engine = ParseEngine(registry)

        elem = etree.fromstring("<parent><unknown>content</unknown></parent>")

        with pytest.raises(UnknownElementError, match="unknown"):
            engine.parse(elem, ParseContext())


class TestParseContext:
    """Tests for ParseContext dataclass."""

    def test_default_values(self) -> None:
        """Test default context values."""
        context = ParseContext()
        assert context.collector is None
        assert context.bwb_id == ""
        assert context.date == ""
        assert context.number_parts == []
        assert context.base_url == ""

    def test_with_values(self) -> None:
        """Test context with provided values."""
        context = ParseContext(
            bwb_id="BWBR0018451",
            date="2025-01-01",
            number_parts=["1", "2"],
            base_url="https://example.com",
        )
        assert context.bwb_id == "BWBR0018451"
        assert context.date == "2025-01-01"
        assert context.number_parts == ["1", "2"]
        assert context.base_url == "https://example.com"

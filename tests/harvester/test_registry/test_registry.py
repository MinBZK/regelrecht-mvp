"""Tests for ElementRegistry."""

from lxml import etree

from harvester.parsers.registry import (
    ElementRegistry,
    ElementType,
    ParseContext,
    ParseResult,
)


class MockHandler:
    """Mock handler for testing."""

    def __init__(self, text: str = "mock text") -> None:
        self._text = text

    @property
    def element_type(self) -> ElementType:
        return ElementType.INLINE

    def can_handle(self, elem: etree._Element, context: ParseContext) -> bool:
        return True

    def handle(self, elem, context, recurse) -> ParseResult:
        return ParseResult(text=self._text)


class ConditionalHandler:
    """Handler that only handles elements with specific attribute."""

    @property
    def element_type(self) -> ElementType:
        return ElementType.INLINE

    def can_handle(self, elem: etree._Element, context: ParseContext) -> bool:
        return elem.get("type") == "special"

    def handle(self, elem, context, recurse) -> ParseResult:
        return ParseResult(text="special")


class TestElementRegistry:
    """Tests for ElementRegistry class."""

    def test_register_and_get_handler(self) -> None:
        """Test registering and retrieving a handler."""
        registry = ElementRegistry()
        handler = MockHandler()
        registry.register("test", handler)

        elem = etree.fromstring("<test>content</test>")
        context = ParseContext()

        result = registry.get_handler(elem, context)
        assert result is handler

    def test_get_handler_returns_none_for_unregistered(self) -> None:
        """Test that unregistered elements return None."""
        registry = ElementRegistry()

        elem = etree.fromstring("<unknown>content</unknown>")
        context = ParseContext()

        result = registry.get_handler(elem, context)
        assert result is None

    def test_skip_tags(self) -> None:
        """Test that skip tags are handled correctly."""
        registry = ElementRegistry()
        registry.skip("meta-data", "kop")

        assert registry.should_skip("meta-data")
        assert registry.should_skip("kop")
        assert not registry.should_skip("al")

    def test_get_handler_returns_none_for_skipped(self) -> None:
        """Test that skipped tags return None from get_handler."""
        registry = ElementRegistry()
        registry.skip("meta-data")
        registry.register("meta-data", MockHandler())  # Register anyway

        elem = etree.fromstring("<meta-data>content</meta-data>")
        context = ParseContext()

        # Should return None because it's in skip list
        result = registry.get_handler(elem, context)
        assert result is None

    def test_has_handler(self) -> None:
        """Test has_handler method."""
        registry = ElementRegistry()
        registry.register("al", MockHandler())

        assert registry.has_handler("al")
        assert not registry.has_handler("unknown")

    def test_registered_tags(self) -> None:
        """Test registered_tags method."""
        registry = ElementRegistry()
        registry.register("al", MockHandler())
        registry.register("nadruk", MockHandler())

        tags = registry.registered_tags()
        assert tags == {"al", "nadruk"}

    def test_skipped_tags(self) -> None:
        """Test skipped_tags method returns a copy."""
        registry = ElementRegistry()
        registry.skip("meta-data", "kop")

        tags = registry.skipped_tags()
        assert tags == {"meta-data", "kop"}

        # Modifying returned set shouldn't affect registry
        tags.add("other")
        assert not registry.should_skip("other")

    def test_conditional_handler_can_handle(self) -> None:
        """Test that can_handle is respected."""
        registry = ElementRegistry()
        registry.register("item", ConditionalHandler())

        context = ParseContext()

        # Element with matching attribute
        special_elem = etree.fromstring('<item type="special">content</item>')
        handler = registry.get_handler(special_elem, context)
        assert handler is not None

        # Element without matching attribute
        normal_elem = etree.fromstring('<item type="normal">content</item>')
        handler = registry.get_handler(normal_elem, context)
        assert handler is None

    def test_namespace_stripped_from_tag(self) -> None:
        """Test that namespace prefixes are stripped from tags."""
        registry = ElementRegistry()
        registry.register("test", MockHandler())

        # Element with namespace
        elem = etree.fromstring(
            '<ns:test xmlns:ns="http://example.com">content</ns:test>'
        )
        context = ParseContext()

        # Should still find handler despite namespace
        handler = registry.get_handler(elem, context)
        assert handler is not None

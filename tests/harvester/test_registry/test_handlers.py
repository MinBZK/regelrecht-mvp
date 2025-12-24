"""Tests for element handlers."""

from lxml import etree

from harvester.parsers.content_parser import ReferenceCollector
from harvester.parsers.registry import ElementType, ParseContext, ParseResult
from harvester.parsers.registry.handlers.inline import (
    AlHandler,
    ExtrefHandler,
    IntrefHandler,
    NadrukHandler,
)
from harvester.parsers.registry.handlers.preamble import (
    ConsideransAlHandler,
    WijHandler,
)
from harvester.parsers.registry.handlers.structural import (
    LidnrHandler,
    LiNrHandler,
    PassthroughHandler,
    SkipHandler,
)


def mock_recurse(elem: etree._Element, context: ParseContext) -> ParseResult:
    """Simple mock recurse that returns element text."""
    text = elem.text or ""
    return ParseResult(text=text)


class TestNadrukHandler:
    """Tests for NadrukHandler."""

    def test_bold_formatting(self) -> None:
        """Test bold (vet) formatting."""
        handler = NadrukHandler()
        elem = etree.fromstring('<nadruk type="vet">important</nadruk>')
        context = ParseContext()

        result = handler.handle(elem, context, mock_recurse)
        assert result.text == "**important**"

    def test_italic_formatting(self) -> None:
        """Test italic (default) formatting."""
        handler = NadrukHandler()
        elem = etree.fromstring("<nadruk>emphasis</nadruk>")
        context = ParseContext()

        result = handler.handle(elem, context, mock_recurse)
        assert result.text == "*emphasis*"

    def test_cursief_formatting(self) -> None:
        """Test cursief type also produces italic."""
        handler = NadrukHandler()
        elem = etree.fromstring('<nadruk type="cur">cursive</nadruk>')
        context = ParseContext()

        result = handler.handle(elem, context, mock_recurse)
        assert result.text == "*cursive*"

    def test_element_type(self) -> None:
        """Test handler element type."""
        handler = NadrukHandler()
        assert handler.element_type == ElementType.INLINE

    def test_can_handle(self) -> None:
        """Test can_handle returns True."""
        handler = NadrukHandler()
        elem = etree.fromstring("<nadruk>text</nadruk>")
        assert handler.can_handle(elem, ParseContext())


class TestExtrefHandler:
    """Tests for ExtrefHandler."""

    def test_with_collector(self) -> None:
        """Test extref with reference collector."""
        handler = ExtrefHandler()
        collector = ReferenceCollector()
        context = ParseContext(collector=collector)

        elem = etree.fromstring(
            '<extref doc="jci1.3:c:BWBR0018450&amp;artikel=1" '
            'bwb-id="BWBR0018450">artikel 1</extref>'
        )

        result = handler.handle(elem, context, mock_recurse)
        assert result.text == "[artikel 1][ref1]"
        assert len(collector.references) == 1
        assert collector.references[0].bwb_id == "BWBR0018450"

    def test_without_collector(self) -> None:
        """Test extref without collector returns inline link."""
        handler = ExtrefHandler()
        context = ParseContext()

        elem = etree.fromstring(
            '<extref doc="jci1.3:c:BWBR0018450&amp;artikel=1">artikel 1</extref>'
        )

        result = handler.handle(elem, context, mock_recurse)
        assert "[artikel 1]" in result.text
        assert "BWBR0018450" in result.text

    def test_without_url(self) -> None:
        """Test extref without doc attribute."""
        handler = ExtrefHandler()
        context = ParseContext()

        elem = etree.fromstring("<extref>plain text</extref>")

        result = handler.handle(elem, context, mock_recurse)
        assert result.text == "plain text"


class TestIntrefHandler:
    """Tests for IntrefHandler."""

    def test_with_collector(self) -> None:
        """Test intref with reference collector."""
        handler = IntrefHandler()
        collector = ReferenceCollector()
        context = ParseContext(collector=collector)

        elem = etree.fromstring(
            '<intref doc="jci1.3:c:BWBR0018451&amp;artikel=2" '
            'bwb-id="BWBR0018451">artikel 2</intref>'
        )

        result = handler.handle(elem, context, mock_recurse)
        assert result.text == "[artikel 2][ref1]"
        assert len(collector.references) == 1


class TestAlHandler:
    """Tests for AlHandler (paragraph)."""

    def test_simple_text(self) -> None:
        """Test simple text extraction."""
        handler = AlHandler()
        elem = etree.fromstring("<al>Simple paragraph text.</al>")
        context = ParseContext()

        result = handler.handle(elem, context, mock_recurse)
        assert result.text == "Simple paragraph text."

    def test_with_children(self) -> None:
        """Test text with child elements."""
        handler = AlHandler()
        elem = etree.fromstring("<al>Text with <child>child</child> content.</al>")
        context = ParseContext()

        result = handler.handle(elem, context, mock_recurse)
        assert "Text with" in result.text
        assert "child" in result.text
        assert "content." in result.text

    def test_preserves_tail_text(self) -> None:
        """Test that tail text after children is preserved."""
        handler = AlHandler()
        elem = etree.fromstring("<al>Before <em>middle</em> after</al>")
        context = ParseContext()

        result = handler.handle(elem, context, mock_recurse)
        assert "Before" in result.text
        assert "after" in result.text


class TestSkipHandler:
    """Tests for SkipHandler."""

    def test_returns_empty(self) -> None:
        """Test that SkipHandler returns empty text."""
        handler = SkipHandler()
        elem = etree.fromstring("<anything>content</anything>")
        context = ParseContext()

        result = handler.handle(elem, context, mock_recurse)
        assert result.text == ""

    def test_element_type_is_skip(self) -> None:
        """Test element type is SKIP."""
        handler = SkipHandler()
        assert handler.element_type == ElementType.SKIP


class TestPassthroughHandler:
    """Tests for PassthroughHandler."""

    def test_extracts_text(self) -> None:
        """Test text extraction."""
        handler = PassthroughHandler()
        elem = etree.fromstring("<container>Hello world</container>")
        context = ParseContext()

        result = handler.handle(elem, context, mock_recurse)
        assert result.text == "Hello world"

    def test_with_children(self) -> None:
        """Test with child elements."""
        handler = PassthroughHandler()
        elem = etree.fromstring("<container>Start <item>middle</item> end</container>")
        context = ParseContext()

        result = handler.handle(elem, context, mock_recurse)
        assert "Start" in result.text
        assert "middle" in result.text
        assert "end" in result.text


class TestLidnrHandler:
    """Tests for LidnrHandler."""

    def test_extracts_number(self) -> None:
        """Test lid number extraction."""
        handler = LidnrHandler()
        elem = etree.fromstring("<lidnr>3</lidnr>")
        context = ParseContext()

        result = handler.handle(elem, context, mock_recurse)
        assert result.text == "3"

    def test_strips_whitespace(self) -> None:
        """Test whitespace stripping."""
        handler = LidnrHandler()
        elem = etree.fromstring("<lidnr>  5  </lidnr>")
        context = ParseContext()

        result = handler.handle(elem, context, mock_recurse)
        assert result.text == "5"


class TestLiNrHandler:
    """Tests for LiNrHandler."""

    def test_extracts_marker(self) -> None:
        """Test list item marker extraction."""
        handler = LiNrHandler()
        elem = etree.fromstring("<li.nr>a.</li.nr>")
        context = ParseContext()

        result = handler.handle(elem, context, mock_recurse)
        assert result.text == "a"

    def test_removes_trailing_period(self) -> None:
        """Test trailing period removal."""
        handler = LiNrHandler()
        elem = etree.fromstring("<li.nr>1.</li.nr>")
        context = ParseContext()

        result = handler.handle(elem, context, mock_recurse)
        assert result.text == "1"

    def test_no_period(self) -> None:
        """Test marker without period."""
        handler = LiNrHandler()
        elem = etree.fromstring("<li.nr>b</li.nr>")
        context = ParseContext()

        result = handler.handle(elem, context, mock_recurse)
        assert result.text == "b"


class TestWijHandler:
    """Tests for WijHandler (royal introduction)."""

    def test_extracts_text(self) -> None:
        """Test royal introduction extraction."""
        handler = WijHandler()
        elem = etree.fromstring(
            "<wij>Wij Beatrix, bij de gratie Gods, Koningin der Nederlanden</wij>"
        )
        context = ParseContext()

        result = handler.handle(elem, context, mock_recurse)
        assert "Wij Beatrix" in result.text
        assert "Koningin der Nederlanden" in result.text


class TestConsideransAlHandler:
    """Tests for ConsideransAlHandler."""

    def test_extracts_text(self) -> None:
        """Test considerans paragraph extraction."""
        handler = ConsideransAlHandler()
        elem = etree.fromstring(
            "<considerans.al>Allen, die deze zullen zien of horen lezen</considerans.al>"
        )
        context = ParseContext()

        result = handler.handle(elem, context, mock_recurse)
        assert "Allen, die deze zullen zien" in result.text

    def test_with_extref(self) -> None:
        """Test considerans with external reference."""
        handler = ConsideransAlHandler()
        elem = etree.fromstring(
            "<considerans.al>Gelet op <extref>wet</extref> foo</considerans.al>"
        )
        context = ParseContext()

        result = handler.handle(elem, context, mock_recurse)
        assert "Gelet op" in result.text
        assert "foo" in result.text

"""Tests for article splitter module."""

from pathlib import Path

from lxml import etree

from harvester.parsers.article_splitter import (
    ArticleComponent,
    build_articles_from_content,
)
from harvester.parsers.text_extractor import (
    extract_inline_text,
    extract_li_text,
    get_intro_text,
    get_li_nr,
    get_lid_nr,
    has_lijst,
)

FIXTURES_DIR = Path(__file__).parent / "fixtures"


class TestArticleComponent:
    """Tests for ArticleComponent dataclass."""

    def test_to_number_single_part(self) -> None:
        """Single number part returns just that number."""
        component = ArticleComponent(
            number_parts=["3"],
            text="Test text",
            base_url="https://example.com",
        )
        assert component.to_number() == "3"

    def test_to_number_multiple_parts(self) -> None:
        """Multiple parts joined with dots."""
        component = ArticleComponent(
            number_parts=["1", "1", "a"],
            text="Test text",
            base_url="https://example.com",
        )
        assert component.to_number() == "1.1.a"

    def test_to_article(self) -> None:
        """Convert to Article object."""
        component = ArticleComponent(
            number_parts=["1", "2"],
            text="Test text",
            base_url="https://example.com#Artikel1",
        )
        article = component.to_article()

        assert article.number == "1.2"
        assert article.text == "Test text"
        assert article.url == "https://example.com#Artikel1"


class TestExtractInlineText:
    """Tests for extract_inline_text function."""

    def test_simple_text(self) -> None:
        """Extract simple text content."""
        elem = etree.fromstring("<al>Simple text content.</al>")
        assert extract_inline_text(elem) == "Simple text content."

    def test_text_with_extref(self) -> None:
        """Extract text with external reference as markdown link."""
        elem = etree.fromstring(
            '<al>See <extref doc="jci1.3:c:BWBR0018450&amp;artikel=1">article 1</extref> for details.</al>'
        )
        result = extract_inline_text(elem)
        assert "[article 1](https://wetten.overheid.nl/BWBR0018450#Artikel1)" in result

    def test_text_with_nadruk_vet(self) -> None:
        """Extract text with bold emphasis."""
        elem = etree.fromstring(
            '<al>This is <nadruk type="vet">bold</nadruk> text.</al>'
        )
        result = extract_inline_text(elem)
        assert "**bold**" in result

    def test_text_with_nadruk_cursief(self) -> None:
        """Extract text with italic emphasis."""
        elem = etree.fromstring(
            '<al>This is <nadruk type="cur">italic</nadruk> text.</al>'
        )
        result = extract_inline_text(elem)
        assert "*italic*" in result

    def test_skips_metadata(self) -> None:
        """Skip meta-data elements."""
        elem = etree.fromstring("<al>Text<meta-data>ignored</meta-data></al>")
        assert extract_inline_text(elem) == "Text"


class TestHelperFunctions:
    """Tests for helper functions."""

    def test_get_li_nr_with_period(self) -> None:
        """Get li.nr and strip trailing period."""
        li = etree.fromstring("<li><li.nr>a.</li.nr><al>Text</al></li>")
        assert get_li_nr(li) == "a"

    def test_get_li_nr_without_period(self) -> None:
        """Get li.nr without period."""
        li = etree.fromstring("<li><li.nr>1</li.nr><al>Text</al></li>")
        assert get_li_nr(li) == "1"

    def test_get_li_nr_missing(self) -> None:
        """Return empty string if no li.nr."""
        li = etree.fromstring("<li><al>Text</al></li>")
        assert get_li_nr(li) == ""

    def test_get_lid_nr(self) -> None:
        """Get lidnr from lid element."""
        lid = etree.fromstring("<lid><lidnr>2</lidnr><al>Text</al></lid>")
        assert get_lid_nr(lid) == "2"

    def test_get_lid_nr_missing(self) -> None:
        """Return empty string if no lidnr."""
        lid = etree.fromstring("<lid><al>Text</al></lid>")
        assert get_lid_nr(lid) == ""

    def test_has_lijst_true(self) -> None:
        """Detect presence of lijst."""
        lid = etree.fromstring("<lid><lijst><li><li.nr>a.</li.nr></li></lijst></lid>")
        assert has_lijst(lid) is True

    def test_has_lijst_false(self) -> None:
        """No lijst present."""
        lid = etree.fromstring("<lid><al>Simple text</al></lid>")
        assert has_lijst(lid) is False

    def test_extract_li_text(self) -> None:
        """Extract text from list item."""
        li = etree.fromstring(
            "<li><li.nr>a.</li.nr><al>Definition text here.</al></li>"
        )
        assert extract_li_text(li) == "Definition text here."

    def test_extract_li_text_multiple_al(self) -> None:
        """Combine text from multiple al elements."""
        li = etree.fromstring(
            "<li><li.nr>a.</li.nr><al>First part.</al><al>Second part.</al></li>"
        )
        assert extract_li_text(li) == "First part. Second part."

    def test_get_intro_text(self) -> None:
        """Extract intro text before lijst."""
        lid = etree.fromstring(
            """<lid>
                <lidnr>1</lidnr>
                <al>In deze wet wordt verstaan onder:</al>
                <lijst><li><li.nr>a.</li.nr><al>item</al></li></lijst>
            </lid>"""
        )
        assert get_intro_text(lid) == "In deze wet wordt verstaan onder:"


class TestBuildArticlesFromContent:
    """Tests for build_articles_from_content function."""

    def test_with_zorgtoeslag_fixture(self) -> None:
        """Integration test with real zorgtoeslag XML fixture."""
        content_path = FIXTURES_DIR / "zorgtoeslag/content.xml"
        content_tree = etree.parse(str(content_path))

        articles = build_articles_from_content(
            content_tree.getroot(), "BWBR0018451", "2025-01-01"
        )

        # Verify we get more articles than the original 8
        # Artikel 1 alone should produce: 1.1 (intro) + 1.1.a-g (7 items) + 1.2 = 9
        assert len(articles) > 8

        # Check article 1.1 (intro text)
        art_1_1 = next((a for a in articles if a.number == "1.1"), None)
        assert art_1_1 is not None
        assert "In deze wet" in art_1_1.text

        # Check article 1.1.a (first onderdeel)
        art_1_1_a = next((a for a in articles if a.number == "1.1.a"), None)
        assert art_1_1_a is not None
        assert "Onze Minister" in art_1_1_a.text

        # Check article 1.2 (second lid without list)
        art_1_2 = next((a for a in articles if a.number == "1.2"), None)
        assert art_1_2 is not None
        assert "draagkracht" in art_1_2.text

        # Check articles with leden (article 3 has 5 leden in 2025 version)
        art_3_1 = next((a for a in articles if a.number == "3.1"), None)
        assert art_3_1 is not None
        assert "rendementsgrondslag" in art_3_1.text.lower()

        art_7 = next((a for a in articles if a.number == "7"), None)
        assert art_7 is not None
        assert "koninklijk besluit" in art_7.text.lower()

        art_8 = next((a for a in articles if a.number == "8"), None)
        assert art_8 is not None
        assert "zorgtoeslag" in art_8.text.lower()

    def test_article_urls(self) -> None:
        """Verify article URLs are correctly generated."""
        content_path = FIXTURES_DIR / "zorgtoeslag/content.xml"
        content_tree = etree.parse(str(content_path))

        articles = build_articles_from_content(
            content_tree.getroot(), "BWBR0018451", "2025-01-01"
        )

        # All components of article 1 should have same base URL
        art_1_articles = [a for a in articles if a.number.startswith("1.")]
        for article in art_1_articles:
            assert (
                article.url
                == "https://wetten.overheid.nl/BWBR0018451/2025-01-01#Artikel1"
            )

    def test_article_order_preserved(self) -> None:
        """Verify articles are in order."""
        content_path = FIXTURES_DIR / "zorgtoeslag/content.xml"
        content_tree = etree.parse(str(content_path))

        articles = build_articles_from_content(
            content_tree.getroot(), "BWBR0018451", "2025-01-01"
        )

        # Get all top-level article numbers
        article_numbers = [a.number.split(".")[0] for a in articles]

        # Should see 1, 1, 1, ... 2, 2, 2, ... 3, 4, ... 8
        seen_articles: list[str] = []
        for nr in article_numbers:
            if nr not in seen_articles:
                seen_articles.append(nr)

        assert seen_articles == ["aanhef", "1", "2", "3", "4", "4a", "5", "6", "7", "8"]

    def test_zorgtoeslag_article_count(self) -> None:
        """Count expected articles from zorgtoeslag."""
        content_path = FIXTURES_DIR / "zorgtoeslag/content.xml"
        content_tree = etree.parse(str(content_path))

        articles = build_articles_from_content(
            content_tree.getroot(), "BWBR0018451", "2025-01-01"
        )

        # Count by article prefix
        art1_count = len([a for a in articles if a.number.startswith("1.")])
        art2_count = len([a for a in articles if a.number.startswith("2.")])

        # Article 1: lid 1 (intro + 8 items a-h) + lid 2 = 10
        assert art1_count == 10

        # Article 2: 7 leden, no lists = 7
        assert art2_count == 7

        # Simple articles (no dot in number): aanhef, 4, 6, 7, 8 = 5
        # Note: Article 3 now has 5 leden, Article 4a has 4 leden, Article 5 has 5 leden
        simple_articles = [a for a in articles if "." not in a.number]
        assert len(simple_articles) == 5

    def test_section_based_article_numbering(self) -> None:
        """Test article numbers with letters and spaces like 'A 1', 'B 5'."""
        xml = """
        <wetgeving>
            <wet-besluit>
                <wettekst>
                    <artikel label="Artikel A 1">
                        <kop><nr>A 1</nr></kop>
                        <al>First article in section A.</al>
                    </artikel>
                    <artikel label="Artikel B 1">
                        <kop><nr>B 1</nr></kop>
                        <lid>
                            <lidnr>1</lidnr>
                            <al>First paragraph of B 1.</al>
                        </lid>
                        <lid>
                            <lidnr>2</lidnr>
                            <al>Second paragraph of B 1.</al>
                        </lid>
                    </artikel>
                </wettekst>
            </wet-besluit>
        </wetgeving>
        """
        root = etree.fromstring(xml)
        articles = build_articles_from_content(root, "BWBR0001234", "2025-01-01")

        # A 1 has no leden, so just one article
        art_a1 = next((a for a in articles if a.number == "A 1"), None)
        assert art_a1 is not None
        assert "First article in section A" in art_a1.text

        # B 1 has leden, so split into B 1.1 and B 1.2
        art_b1_1 = next((a for a in articles if a.number == "B 1.1"), None)
        art_b1_2 = next((a for a in articles if a.number == "B 1.2"), None)
        assert art_b1_1 is not None
        assert art_b1_2 is not None
        assert "First paragraph" in art_b1_1.text
        assert "Second paragraph" in art_b1_2.text

        # URL should use underscores instead of spaces
        assert "ArtikelA_1" in art_a1.url
        assert "ArtikelB_1" in art_b1_1.url

    def test_deep_nesting(self) -> None:
        """Test deeply nested structures like artikel 1.1, lid 2, onderdeel 1, sub a."""
        xml = """
        <wetgeving>
            <wet-besluit>
                <wettekst>
                    <artikel label="Artikel 1.1">
                        <kop><nr>1.1</nr></kop>
                        <lid>
                            <lidnr>1</lidnr>
                            <al>Simple first paragraph.</al>
                        </lid>
                        <lid>
                            <lidnr>2</lidnr>
                            <al>Intro text for list:</al>
                            <lijst type="expliciet">
                                <li>
                                    <li.nr>1.</li.nr>
                                    <al>Nested intro:</al>
                                    <lijst type="expliciet">
                                        <li>
                                            <li.nr>a.</li.nr>
                                            <al>Deepest level item a.</al>
                                        </li>
                                        <li>
                                            <li.nr>b.</li.nr>
                                            <al>Deepest level item b.</al>
                                        </li>
                                    </lijst>
                                </li>
                            </lijst>
                        </lid>
                    </artikel>
                </wettekst>
            </wet-besluit>
        </wetgeving>
        """
        root = etree.fromstring(xml)
        articles = build_articles_from_content(root, "BWBR0001234", "2025-01-01")

        # Check deep nesting produces correct article numbers
        art_1_1_1 = next((a for a in articles if a.number == "1.1.1"), None)
        art_1_1_2 = next((a for a in articles if a.number == "1.1.2"), None)
        art_1_1_2_1 = next((a for a in articles if a.number == "1.1.2.1"), None)
        art_1_1_2_1_a = next((a for a in articles if a.number == "1.1.2.1.a"), None)
        art_1_1_2_1_b = next((a for a in articles if a.number == "1.1.2.1.b"), None)

        assert art_1_1_1 is not None, "Missing 1.1.1"
        assert art_1_1_2 is not None, "Missing 1.1.2 (intro)"
        assert art_1_1_2_1 is not None, "Missing 1.1.2.1 (nested intro)"
        assert art_1_1_2_1_a is not None, "Missing 1.1.2.1.a"
        assert art_1_1_2_1_b is not None, "Missing 1.1.2.1.b"

        assert "Simple first" in art_1_1_1.text
        assert "Deepest level item a" in art_1_1_2_1_a.text
        assert "Deepest level item b" in art_1_1_2_1_b.text

    def test_unmarked_lists_inline(self) -> None:
        """Test that unmarked lists (type='ongemarkeerd') are rendered inline as markdown."""
        xml = """
        <wetgeving>
            <wet-besluit>
                <wettekst>
                    <artikel label="Artikel 1">
                        <kop><nr>1</nr></kop>
                        <al>In deze wet wordt verstaan onder:</al>
                        <lijst type="ongemarkeerd">
                            <li>
                                <li.nr>–</li.nr>
                                <al>eerste begrip: definitie van eerste begrip;</al>
                            </li>
                            <li>
                                <li.nr>–</li.nr>
                                <al>tweede begrip: definitie van tweede begrip.</al>
                            </li>
                        </lijst>
                    </artikel>
                </wettekst>
            </wet-besluit>
        </wetgeving>
        """
        root = etree.fromstring(xml)
        articles = build_articles_from_content(root, "BWBR0001234", "2025-01-01")

        # Should produce only ONE article, not split by list items
        non_aanhef = [a for a in articles if a.number != "aanhef"]
        assert len(non_aanhef) == 1

        art_1 = non_aanhef[0]
        assert art_1.number == "1"

        # Text should contain markdown bullet points
        assert "- eerste begrip:" in art_1.text
        assert "- tweede begrip:" in art_1.text

    def test_unmarked_list_in_lid(self) -> None:
        """Test unmarked list inside a lid is rendered inline."""
        xml = """
        <wetgeving>
            <wet-besluit>
                <wettekst>
                    <artikel label="Artikel 1">
                        <kop><nr>1</nr></kop>
                        <lid>
                            <lidnr>1</lidnr>
                            <al>Regular paragraph.</al>
                        </lid>
                        <lid>
                            <lidnr>2</lidnr>
                            <al>List intro:</al>
                            <lijst type="ongemarkeerd">
                                <li>
                                    <li.nr>-</li.nr>
                                    <al>item one;</al>
                                </li>
                                <li>
                                    <li.nr>-</li.nr>
                                    <al>item two.</al>
                                </li>
                            </lijst>
                        </lid>
                    </artikel>
                </wettekst>
            </wet-besluit>
        </wetgeving>
        """
        root = etree.fromstring(xml)
        articles = build_articles_from_content(root, "BWBR0001234", "2025-01-01")

        # Should produce 1.1 and 1.2 (not 1.2.1, 1.2.2 for unmarked items)
        numbers = [a.number for a in articles if a.number != "aanhef"]
        assert "1.1" in numbers
        assert "1.2" in numbers
        assert "1.2.1" not in numbers  # Unmarked items not split

        art_1_2 = next(a for a in articles if a.number == "1.2")
        assert "- item one" in art_1_2.text
        assert "- item two" in art_1_2.text

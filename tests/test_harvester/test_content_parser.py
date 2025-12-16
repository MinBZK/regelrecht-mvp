"""Tests for content (legal text) parser."""

from pathlib import Path
from unittest.mock import Mock, patch

import pytest
from lxml import etree

from harvester.parsers.content_parser import (
    BWB_REPOSITORY_URL,
    download_content,
    extract_text_from_element,
    parse_articles,
)

FIXTURES_DIR = Path(__file__).parent / "fixtures"


class TestExtractTextFromElement:
    """Tests for extract_text_from_element function."""

    def test_extract_simple_text(self) -> None:
        """Extract text from simple element."""
        xml = b"<al>Simple text content.</al>"
        elem = etree.fromstring(xml)

        result = extract_text_from_element(elem)

        assert result == "Simple text content."

    def test_extract_none_element(self) -> None:
        """Handle None element."""
        result = extract_text_from_element(None)

        assert result == ""

    def test_extract_nested_paragraphs(self) -> None:
        """Extract text from nested al (paragraph) elements."""
        xml = b"""<artikel>
            <al>First paragraph.</al>
            <al>Second paragraph.</al>
        </artikel>"""
        elem = etree.fromstring(xml)

        result = extract_text_from_element(elem)

        assert "First paragraph." in result
        assert "Second paragraph." in result

    def test_extract_list(self) -> None:
        """Extract text from lijst (list) elements."""
        xml = b"""<artikel>
            <lijst>
                <li><al>Item one</al></li>
                <li><al>Item two</al></li>
            </lijst>
        </artikel>"""
        elem = etree.fromstring(xml)

        result = extract_text_from_element(elem)

        assert "- Item one" in result
        assert "- Item two" in result

    def test_extract_emphasis_bold(self) -> None:
        """Extract bold emphasis (nadruk type=vet)."""
        xml = b'<al>This is <nadruk type="vet">important</nadruk> text.</al>'
        elem = etree.fromstring(xml)

        result = extract_text_from_element(elem)

        assert "**important**" in result

    def test_extract_emphasis_italic(self) -> None:
        """Extract italic emphasis (nadruk without type=vet)."""
        xml = b'<al>This is <nadruk type="cur">emphasized</nadruk> text.</al>'
        elem = etree.fromstring(xml)

        result = extract_text_from_element(elem)

        assert "*emphasized*" in result

    def test_extract_external_reference(self) -> None:
        """Extract external references (extref)."""
        xml = b'<al>See <extref doc="http://example.com">this link</extref> for more.</al>'
        elem = etree.fromstring(xml)

        result = extract_text_from_element(elem)

        assert "[this link](http://example.com)" in result

    def test_extract_external_reference_no_url(self) -> None:
        """Extract external references without doc attribute."""
        xml = b"<al>See <extref>this reference</extref> for more.</al>"
        elem = etree.fromstring(xml)

        result = extract_text_from_element(elem)

        assert "this reference" in result
        assert "[" not in result  # No markdown link

    def test_extract_tail_text(self) -> None:
        """Extract tail text after child elements."""
        xml = b"<al>Before <nadruk>middle</nadruk> after.</al>"
        elem = etree.fromstring(xml)

        result = extract_text_from_element(elem)

        assert "Before" in result
        assert "middle" in result
        assert "after." in result


class TestParseArticles:
    """Tests for parse_articles function."""

    def test_parse_standard_articles(self) -> None:
        """Parse articles from standard content file."""
        xml_path = FIXTURES_DIR / "sample_toestand.xml"
        tree = etree.parse(str(xml_path))
        root = tree.getroot()

        articles = parse_articles(root, "BWBR0018451", "2025-01-01")

        assert len(articles) == 3
        assert articles[0].number == "1"
        assert articles[1].number == "2"
        assert articles[2].number == "3"

    def test_parse_article_text_content(self) -> None:
        """Verify article text is extracted correctly."""
        xml_path = FIXTURES_DIR / "sample_toestand.xml"
        tree = etree.parse(str(xml_path))
        root = tree.getroot()

        articles = parse_articles(root, "BWBR0018451", "2025-01-01")

        # Article 1 has a list
        assert "In deze wet wordt verstaan onder:" in articles[0].text
        assert "- a. Onze Minister:" in articles[0].text

        # Article 2 has bold text
        assert "**de draagkracht**" in articles[1].text

        # Article 3 has an external reference
        assert "[artikel 1 van de Zorgverzekeringswet]" in articles[2].text

    def test_parse_article_url_generation(self) -> None:
        """Verify article URLs are generated correctly."""
        xml_path = FIXTURES_DIR / "sample_toestand.xml"
        tree = etree.parse(str(xml_path))
        root = tree.getroot()

        articles = parse_articles(root, "BWBR0018451", "2025-01-01")

        assert (
            articles[0].url
            == "https://wetten.overheid.nl/BWBR0018451/2025-01-01#Artikel1"
        )
        assert (
            articles[1].url
            == "https://wetten.overheid.nl/BWBR0018451/2025-01-01#Artikel2"
        )
        assert (
            articles[2].url
            == "https://wetten.overheid.nl/BWBR0018451/2025-01-01#Artikel3"
        )

    def test_parse_article_fallback_to_label(self) -> None:
        """Extract article number from label when nr element is missing."""
        xml_path = FIXTURES_DIR / "sample_toestand_no_nr.xml"
        tree = etree.parse(str(xml_path))
        root = tree.getroot()

        articles = parse_articles(root, "BWBR0099999", "2025-01-01")

        # Only article with label should be parsed
        assert len(articles) == 1
        assert articles[0].number == "1"

    def test_parse_article_skip_no_number(self) -> None:
        """Skip articles without number or label."""
        xml_content = b"""<?xml version="1.0"?>
        <wetgeving>
            <wet>
                <wettekst>
                    <artikel>
                        <al>No number article.</al>
                    </artikel>
                </wettekst>
            </wet>
        </wetgeving>"""
        root = etree.fromstring(xml_content)

        articles = parse_articles(root, "BWBR0099999", "2025-01-01")

        assert len(articles) == 0

    def test_parse_empty_content(self) -> None:
        """Handle empty content file."""
        xml_content = b"""<?xml version="1.0"?>
        <wetgeving>
            <wet>
                <wettekst></wettekst>
            </wet>
        </wetgeving>"""
        root = etree.fromstring(xml_content)

        articles = parse_articles(root, "BWBR0099999", "2025-01-01")

        assert len(articles) == 0


class TestDownloadContent:
    """Tests for download_content function."""

    def test_download_constructs_correct_url(self) -> None:
        """Verify correct URL construction."""
        with patch("harvester.parsers.content_parser.requests.get") as mock_get:
            mock_response = Mock()
            mock_response.content = b"""<?xml version="1.0"?>
                <wetgeving><wet><wettekst></wettekst></wet></wetgeving>"""
            mock_response.raise_for_status = Mock()
            mock_get.return_value = mock_response

            download_content("BWBR0018451", "2025-01-01")

            # URL should use _0 suffix for consolidated version
            mock_get.assert_called_once_with(
                f"{BWB_REPOSITORY_URL}/BWBR0018451/2025-01-01_0/xml/BWBR0018451_2025-01-01_0.xml",
                timeout=30,
                allow_redirects=True,
            )

    def test_download_raises_on_http_error(self) -> None:
        """HTTP errors should propagate."""
        with patch("harvester.parsers.content_parser.requests.get") as mock_get:
            from requests.exceptions import HTTPError

            mock_response = Mock()
            mock_response.raise_for_status.side_effect = HTTPError("404 Not Found")
            mock_get.return_value = mock_response

            with pytest.raises(HTTPError):
                download_content("BWBR9999999", "2025-01-01")

    def test_download_returns_element(self) -> None:
        """Download should return an lxml Element."""
        with patch("harvester.parsers.content_parser.requests.get") as mock_get:
            mock_response = Mock()
            mock_response.content = b"""<?xml version="1.0"?>
                <wetgeving><wet><wettekst></wettekst></wet></wetgeving>"""
            mock_response.raise_for_status = Mock()
            mock_get.return_value = mock_response

            result = download_content("BWBR0018451", "2025-01-01")

            assert isinstance(result, etree._Element)
            assert result.tag == "wetgeving"

"""Tests for WTI (metadata) parser."""

from pathlib import Path
from unittest.mock import Mock, patch

import pytest
from lxml import etree

from harvester.models import RegulatoryLayer
from harvester.parsers.wti_parser import (
    BWB_REPOSITORY_URL,
    download_wti,
    parse_wti_metadata,
)

FIXTURES_DIR = Path(__file__).parent / "fixtures"


class TestParseWtiMetadata:
    """Tests for parse_wti_metadata function."""

    def test_parse_standard_wti(self) -> None:
        """Parse a standard WTI file with official title."""
        xml_path = FIXTURES_DIR / "sample_wti.xml"
        tree = etree.parse(str(xml_path))
        root = tree.getroot()

        metadata = parse_wti_metadata(root)

        assert metadata.bwb_id == "BWBR0018451"
        assert metadata.title == "Wet op de zorgtoeslag"
        assert metadata.regulatory_layer == RegulatoryLayer.WET
        assert metadata.publication_date == "2005-07-26"

    def test_parse_amvb(self) -> None:
        """Parse WTI for an AMVB (algemene maatregel van bestuur)."""
        xml_path = FIXTURES_DIR / "sample_wti_amvb.xml"
        tree = etree.parse(str(xml_path))
        root = tree.getroot()

        metadata = parse_wti_metadata(root)

        assert metadata.bwb_id == "BWBR0012345"
        assert metadata.title == "Besluit zorgtoeslag"
        assert metadata.regulatory_layer == RegulatoryLayer.AMVB
        assert metadata.publication_date == "2010-01-01"

    def test_parse_fallback_to_unofficial_title(self) -> None:
        """Fallback to any citeertitel if no official one exists."""
        xml_path = FIXTURES_DIR / "sample_wti_no_official.xml"
        tree = etree.parse(str(xml_path))
        root = tree.getroot()

        metadata = parse_wti_metadata(root)

        assert metadata.title == "Informele titel wet"
        assert metadata.publication_date is None

    def test_parse_ministeriele_regeling(self) -> None:
        """Parse WTI for a ministeriele regeling."""
        xml_content = b"""<?xml version="1.0" encoding="UTF-8"?>
        <wti bwb-id="BWBR0099999">
          <titel>
            <citeertitel status="officieel">Regeling zorgtoeslag</citeertitel>
          </titel>
          <soort-regeling>ministeri\xc3\xable regeling</soort-regeling>
        </wti>"""
        root = etree.fromstring(xml_content)

        metadata = parse_wti_metadata(root)

        assert metadata.regulatory_layer == RegulatoryLayer.MINISTERIELE_REGELING

    def test_parse_koninklijk_besluit(self) -> None:
        """Parse WTI for a koninklijk besluit."""
        xml_content = b"""<?xml version="1.0" encoding="UTF-8"?>
        <wti bwb-id="BWBR0099999">
          <titel>
            <citeertitel status="officieel">KB Test</citeertitel>
          </titel>
          <soort-regeling>koninklijk besluit</soort-regeling>
        </wti>"""
        root = etree.fromstring(xml_content)

        metadata = parse_wti_metadata(root)

        assert metadata.regulatory_layer == RegulatoryLayer.KONINKLIJK_BESLUIT

    def test_parse_unknown_type_defaults_to_wet(self) -> None:
        """Unknown regulatory type defaults to WET."""
        xml_content = b"""<?xml version="1.0" encoding="UTF-8"?>
        <wti bwb-id="BWBR0099999">
          <titel>
            <citeertitel status="officieel">Onbekend Type</citeertitel>
          </titel>
          <soort-regeling>onbekend type</soort-regeling>
        </wti>"""
        root = etree.fromstring(xml_content)

        metadata = parse_wti_metadata(root)

        assert metadata.regulatory_layer == RegulatoryLayer.WET

    def test_parse_empty_title(self) -> None:
        """Handle empty citeertitel gracefully."""
        xml_content = b"""<?xml version="1.0" encoding="UTF-8"?>
        <wti bwb-id="BWBR0099999">
          <titel>
            <citeertitel status="officieel"></citeertitel>
          </titel>
          <soort-regeling>wet</soort-regeling>
        </wti>"""
        root = etree.fromstring(xml_content)

        metadata = parse_wti_metadata(root)

        assert metadata.title == ""

    def test_parse_no_citeertitel(self) -> None:
        """Handle missing citeertitel."""
        xml_content = b"""<?xml version="1.0" encoding="UTF-8"?>
        <wti bwb-id="BWBR0099999">
          <titel></titel>
          <soort-regeling>wet</soort-regeling>
        </wti>"""
        root = etree.fromstring(xml_content)

        metadata = parse_wti_metadata(root)

        assert metadata.title == ""


class TestDownloadWti:
    """Tests for download_wti function."""

    def test_download_constructs_correct_url(self) -> None:
        """Verify correct URL construction."""
        with patch("harvester.parsers.wti_parser.requests.get") as mock_get:
            mock_response = Mock()
            mock_response.content = b"""<?xml version="1.0"?>
                <wti bwb-id="BWBR0018451">
                  <titel><citeertitel status="officieel">Test</citeertitel></titel>
                  <soort-regeling>wet</soort-regeling>
                </wti>"""
            mock_response.raise_for_status = Mock()
            mock_get.return_value = mock_response

            download_wti("BWBR0018451")

            mock_get.assert_called_once_with(
                f"{BWB_REPOSITORY_URL}/BWBR0018451/BWBR0018451.WTI",
                timeout=30,
            )

    def test_download_raises_on_http_error(self) -> None:
        """HTTP errors should propagate."""
        with patch("harvester.parsers.wti_parser.requests.get") as mock_get:
            from requests.exceptions import HTTPError

            mock_response = Mock()
            mock_response.raise_for_status.side_effect = HTTPError("404 Not Found")
            mock_get.return_value = mock_response

            with pytest.raises(HTTPError):
                download_wti("BWBR9999999")

    def test_download_returns_element(self) -> None:
        """Download should return an lxml Element."""
        with patch("harvester.parsers.wti_parser.requests.get") as mock_get:
            mock_response = Mock()
            mock_response.content = b"""<?xml version="1.0"?>
                <wti bwb-id="BWBR0018451">
                  <titel><citeertitel status="officieel">Test</citeertitel></titel>
                  <soort-regeling>wet</soort-regeling>
                </wti>"""
            mock_response.raise_for_status = Mock()
            mock_get.return_value = mock_response

            result = download_wti("BWBR0018451")

            assert isinstance(result, etree._Element)
            assert result.get("bwb-id") == "BWBR0018451"

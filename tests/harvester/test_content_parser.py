"""Tests for content (legal text) parser."""

from unittest.mock import Mock, patch

import pytest
from lxml import etree

from harvester.parsers.content_parser import BWB_REPOSITORY_URL, download_content


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
                timeout=10,
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

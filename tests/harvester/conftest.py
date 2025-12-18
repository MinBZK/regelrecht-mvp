"""Shared test fixtures for harvester tests."""

from pathlib import Path
from unittest.mock import Mock

import pytest


# Shared fixtures directory
FIXTURES_DIR = Path(__file__).parent / "fixtures"


@pytest.fixture
def fixtures_dir() -> Path:
    """Return the path to the test fixtures directory."""
    return FIXTURES_DIR


@pytest.fixture
def mock_http_response():
    """Factory fixture to create mock HTTP responses.

    Usage:
        def test_example(mock_http_response):
            response = mock_http_response(b"<xml>content</xml>")
            # response.content == b"<xml>content</xml>"
            # response.raise_for_status() does nothing
    """

    def _create_response(content: bytes) -> Mock:
        response = Mock()
        response.content = content
        response.raise_for_status = Mock()
        return response

    return _create_response

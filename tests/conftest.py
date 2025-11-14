"""
Pytest configuration and fixtures for regelrecht-mvp tests
"""
import pytest
from engine.article_loader import Article, ArticleBasedLaw


@pytest.fixture
def sample_law_data():
    """Sample law YAML data for testing"""
    return {
        "$id": "test_law",
        "uuid": "12345678-1234-4234-8234-123456789012",
        "regulatory_layer": "WET",
        "publication_date": "2025-01-01",
        "url": "https://example.com/test",
        "articles": [
            {
                "number": "1",
                "text": "Test article text",
                "url": "https://example.com/test#1",
                "machine_readable": {
                    "public": True,
                    "endpoint": "test_law.test_endpoint",
                    "execution": {
                        "output": [
                            {"name": "test_output", "type": "number"}
                        ],
                        "actions": [
                            {"output": "test_output", "value": 42}
                        ]
                    }
                }
            }
        ]
    }


@pytest.fixture
def minimal_article():
    """Minimal article for testing"""
    return Article({
        "number": "1",
        "text": "Test article",
        "url": "https://example.com/test#1",
        "machine_readable": {
            "execution": {
                "output": [{"name": "result", "type": "number"}],
                "actions": [{"output": "result", "value": 42}]
            }
        }
    })


@pytest.fixture
def minimal_law():
    """Minimal law for testing"""
    return ArticleBasedLaw({
        "$id": "test_law",
        "uuid": "test-uuid-12345",
        "regulatory_layer": "WET",
        "publication_date": "2025-01-01",
        "articles": []
    })

"""
Article-based law loader

Handles loading and parsing of article-based legal specifications.
"""

from dataclasses import dataclass
from typing import Any


@dataclass
class Article:
    """Represents a single article in a law"""

    number: str
    text: str
    url: str
    machine_readable: dict[str, Any]

    def __init__(self, article_data: dict):
        self.number = article_data["number"]
        self.text = article_data["text"]
        # Support both 'url' and 'ref' for backward compatibility
        self.url = article_data.get("url") or article_data.get("ref")
        self.machine_readable = article_data.get("machine_readable", {})

    def get_execution_spec(self) -> dict:
        """Extract execution specification from machine_readable section"""
        return self.machine_readable.get("execution", {})

    def get_definitions(self) -> dict:
        """Get definitions from this article"""
        return self.machine_readable.get("definitions", {})

    def get_requires(self) -> list[str]:
        """Get required URI dependencies"""
        return self.machine_readable.get("requires", [])

    def get_endpoint(self) -> str | None:
        """Get the endpoint name if this article is public"""
        if self.machine_readable.get("public"):
            return self.machine_readable.get("endpoint")
        return None

    def is_public(self) -> bool:
        """Check if this article is publicly callable"""
        return self.machine_readable.get("public", False)

    def get_competent_authority(self) -> str | None:
        """Get the competent authority for this article"""
        return self.machine_readable.get("competent_authority")


@dataclass
class ArticleBasedLaw:
    """Represents an article-based law document"""

    schema: str
    id: str
    uuid: str
    regulatory_layer: str
    publication_date: str
    identifiers: dict[str, str]
    articles: list[Article]

    def __init__(self, yaml_data: dict):
        self.schema = yaml_data.get("$schema", "")
        self.id = yaml_data["$id"]
        self.uuid = yaml_data["uuid"]
        self.regulatory_layer = yaml_data["regulatory_layer"]
        self.publication_date = yaml_data["publication_date"]
        self.effective_date = yaml_data.get("effective_date")
        self.name = yaml_data.get("name")
        self.competent_authority = yaml_data.get("competent_authority")
        self.identifiers = yaml_data.get("identifiers", {})
        self.articles = [Article(art) for art in yaml_data.get("articles", [])]

    def find_article_by_endpoint(self, endpoint: str) -> Article | None:
        """Find article with given endpoint"""
        for article in self.articles:
            if article.get_endpoint() == endpoint:
                return article
        return None

    def find_article_by_number(self, number: str) -> Article | None:
        """Find article by article number"""
        for article in self.articles:
            if article.number == number:
                return article
        return None

    def get_all_endpoints(self) -> dict[str, Article]:
        """Get mapping of endpoint names to articles"""
        endpoints = {}
        for article in self.articles:
            endpoint = article.get_endpoint()
            if endpoint:
                endpoints[endpoint] = article
        return endpoints

    def get_public_articles(self) -> list[Article]:
        """Get all publicly callable articles"""
        return [art for art in self.articles if art.is_public()]

    def get_bwb_id(self) -> str | None:
        """Get BWB identifier if available"""
        return self.identifiers.get("bwb_id")

    def get_url(self) -> str | None:
        """Get official URL if available"""
        return self.identifiers.get("url") or self.identifiers.get("ref")

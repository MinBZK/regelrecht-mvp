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
    url: str | None
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

    def get_output_names(self) -> list[str]:
        """Get all output names from this article - these are the public endpoints"""
        execution = self.machine_readable.get("execution", {})
        outputs = execution.get("output", [])
        return [o.get("name") for o in outputs if o.get("name")]

    def is_public(self) -> bool:
        """Check if this article is publicly callable (has outputs)"""
        return len(self.get_output_names()) > 0

    def get_competent_authority(self) -> str | None:
        """Get the competent authority for this article"""
        return self.machine_readable.get("competent_authority")


@dataclass
class ArticleBasedLaw:
    """Represents an article-based law document"""

    schema: str
    id: str
    uuid: str | None
    regulatory_layer: str
    publication_date: str
    identifiers: dict[str, str]
    articles: list[Article]

    def __init__(self, yaml_data: dict):
        self.schema = yaml_data.get("$schema", "")
        self.id = yaml_data["$id"]
        self.uuid = yaml_data.get("uuid")
        self.regulatory_layer = yaml_data["regulatory_layer"]
        self.publication_date = yaml_data["publication_date"]
        self.valid_from = yaml_data.get("valid_from")
        self.name = yaml_data.get("name")
        self.competent_authority = yaml_data.get("competent_authority")
        self.bwb_id = yaml_data.get("bwb_id")
        self.url = yaml_data.get("url")
        self.identifiers = yaml_data.get("identifiers", {})
        # For gemeentelijke verordeningen
        self.gemeente_code = yaml_data.get("gemeente_code")
        self.officiele_titel = yaml_data.get("officiele_titel")
        # For versioned regulations (e.g., tariffs that change per year)
        self.jaar = yaml_data.get("jaar")
        self.articles = [Article(art) for art in yaml_data.get("articles", [])]

    def find_article_by_output(self, output_name: str) -> Article | None:
        """Find article that produces the given output"""
        for article in self.articles:
            if output_name in article.get_output_names():
                return article
        return None

    def find_article_by_number(self, number: str) -> Article | None:
        """Find article by article number"""
        for article in self.articles:
            if article.number == number:
                return article
        return None

    def get_all_outputs(self) -> dict[str, Article]:
        """Get mapping of output names to articles"""
        outputs = {}
        for article in self.articles:
            for output_name in article.get_output_names():
                outputs[output_name] = article
        return outputs

    def get_public_articles(self) -> list[Article]:
        """Get all publicly callable articles"""
        return [art for art in self.articles if art.is_public()]

    def get_bwb_id(self) -> str | None:
        """Get BWB identifier if available"""
        # Support both top-level bwb_id and identifiers.bwb_id
        return self.bwb_id or self.identifiers.get("bwb_id")

    def get_url(self) -> str | None:
        """Get official URL if available"""
        # Support both top-level url and identifiers.url/ref
        return self.url or self.identifiers.get("url") or self.identifiers.get("ref")

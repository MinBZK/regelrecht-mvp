"""
Core Pydantic models for Dutch legal regulations.

This module provides shared data models for representing laws and articles
that can be used across different projects (regelrecht-mvp, WIAT, etc.).
"""

from pydantic import BaseModel


class Article(BaseModel):
    """
    Law article with text content.

    Attributes:
        number: Article number (e.g., "2", "4a", "37 lid 4")
        text: The verbatim legal text of the article
        url: Optional URL to the official source
    """

    number: str
    text: str
    url: str | None = None


class Law(BaseModel):
    """
    Law document with articles.

    Attributes:
        id: Unique identifier/slug for the law (e.g., "zorgtoeslagwet")
        name: Human-readable name of the law
        bwb_id: BWB identifier (e.g., "BWBR0018451")
        articles: List of articles in this law
    """

    id: str
    name: str | None = None
    bwb_id: str | None = None
    articles: list[Article] = []

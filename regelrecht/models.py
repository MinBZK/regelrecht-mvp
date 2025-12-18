"""
Core Pydantic models for Dutch legal regulations.

This module provides shared data models for representing laws and articles
that can be used across different projects (regelrecht-mvp, WIAT, etc.).
"""

from __future__ import annotations

from pathlib import Path
from typing import Self

import yaml
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

    @classmethod
    def from_yaml(cls, yaml_text: str) -> Self:
        """
        Load a Law from YAML text.

        Handles the $id field mapping and article parsing.

        Example:
            law = Law.from_yaml('''
                $id: zorgtoeslagwet
                bwb_id: BWBR0018451
                articles:
                  - number: "2"
                    text: "De verzekerde heeft aanspraak..."
            ''')
        """
        data = yaml.safe_load(yaml_text)
        return cls._from_dict(data)

    @classmethod
    def from_yaml_file(cls, path: str | Path) -> Self:
        """Load a Law from a YAML file."""
        with open(path) as f:
            data = yaml.safe_load(f)
        return cls._from_dict(data)

    @classmethod
    def _from_dict(cls, data: dict) -> Self:
        """Create Law from parsed YAML dict."""
        articles = [
            Article(
                number=str(a.get("number", "")),
                text=a.get("text", ""),
                url=a.get("url"),
            )
            for a in data.get("articles", [])
        ]
        return cls(
            id=data.get("$id", ""),
            name=data.get("name"),
            bwb_id=data.get("bwb_id"),
            articles=articles,
        )

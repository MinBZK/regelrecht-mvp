"""
regelrecht - Shared library for Dutch legal regulations.

This library provides:
- Core Pydantic models for laws and articles
- W3C Web Annotation TextQuoteSelector for text annotations
- Fuzzy matching for version-resilient annotation resolution

Example usage:
    from regelrecht import Article, Law, TextQuoteSelector, resolve_selector, MatchResult

    # Create law with articles
    law = Law(id="zorgtoeslagwet", articles=[
        Article(number="2", text="...")
    ])

    # Create selector and resolve
    selector = TextQuoteSelector(exact="zorgtoeslag", prefix="op een ")
    result, matches = resolve_selector("", selector, articles=law.articles)
"""

from regelrecht.models import Article, Law
from regelrecht.selectors import Match, MatchResult, TextQuoteSelector, resolve_selector

__all__ = [
    # Core models
    "Article",
    "Law",
    # Annotation/selector
    "TextQuoteSelector",
    "Match",
    "MatchResult",
    "resolve_selector",
]

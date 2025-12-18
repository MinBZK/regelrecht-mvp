"""
regelrecht - Shared library for Dutch legal regulations.

This library provides:
- Core Pydantic models for laws and articles
- W3C Web Annotation TextQuoteSelector for text annotations
- Fuzzy matching for version-resilient annotation resolution

Example usage:
    from regelrecht import TextQuoteSelector, Law, Article

    # Create law with articles
    law = Law(id="zorgtoeslagwet", articles=[
        Article(number="2", text="...")
    ])

    # Create selector and locate
    selector = TextQuoteSelector(exact="zorgtoeslag", prefix="op een ")
    result = selector.locate(law)

    if result.found:
        print(result.match.article_number)
"""

from regelrecht.models import Article, Law
from regelrecht.selectors import Match, MatchResult, MatchStatus, TextQuoteSelector

__all__ = [
    # Core models
    "Article",
    "Law",
    # Selector and matching
    "TextQuoteSelector",
    "Match",
    "MatchResult",
    "MatchStatus",
]

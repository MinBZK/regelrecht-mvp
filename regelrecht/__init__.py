"""
regelrecht - Shared library for Dutch legal regulations.

This library provides:
- Core Pydantic models for laws and articles
- W3C Web Annotation TextQuoteSelector for text annotations
- Fuzzy matching for version-resilient annotation resolution

Import patterns:

    # Primary API (recommended)
    from regelrecht import TextQuoteSelector, Article, Law

    # Full submodule imports (for internal types)
    from regelrecht.models import Article, Law
    from regelrecht.selectors import TextQuoteSelector, MatchResult, Match, MatchStatus

Example usage:

    from regelrecht import TextQuoteSelector, Article

    # Create selector
    selector = TextQuoteSelector(exact="zorgtoeslag", prefix="op een ")

    # Locate in articles
    articles = [Article(number="2", text="...zorgtoeslag...")]
    result = selector.locate(articles)

    if result.found:
        print(result.match.article_number)
    elif result.ambiguous:
        print(f"{len(result.matches)} matches found")
"""

from regelrecht.models import Article, Law
from regelrecht.selectors import MatchResult, MatchStatus, TextQuoteSelector

# Primary public API
__all__ = [
    "Article",
    "Law",
    "TextQuoteSelector",
    "MatchResult",
    "MatchStatus",
]

# Note: For Match (individual match details), use:
#   from regelrecht.selectors import Match

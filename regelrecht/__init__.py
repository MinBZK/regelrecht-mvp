"""
regelrecht - Shared library for Dutch legal regulations.

This library provides:
- Core Pydantic models for laws and articles (regelrecht.models)
- W3C Web Annotation TextQuoteSelector for text annotations (regelrecht.selectors)
- Fuzzy matching for version-resilient annotation resolution

Example usage:

    from regelrecht.models import Article
    from regelrecht.selectors import TextQuoteSelector

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

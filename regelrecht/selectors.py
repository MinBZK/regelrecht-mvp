"""
TextQuoteSelector implementation for W3C Web Annotation.

This module provides the core data structures and resolution algorithm
for TextQuoteSelector-based annotations as specified in RFC-004.
"""

from __future__ import annotations

from enum import Enum
from typing import TYPE_CHECKING, Self

import yaml
from pydantic import BaseModel, computed_field

from regelrecht.matcher import find_fuzzy_matches

if TYPE_CHECKING:
    from regelrecht.models import Article, Law


class MatchStatus(str, Enum):
    """Status of a selector match attempt."""

    FOUND = "found"
    ORPHANED = "orphaned"
    AMBIGUOUS = "ambiguous"


class Hint(BaseModel):
    """
    Performance hint for TextQuoteSelector resolution.

    Specifies where to look first (article number and optional character range).
    If the text isn't found at the hinted location, the entire law is searched.

    Attributes:
        article: Article number to search first (e.g., "2", "4a")
        start: Optional character offset where the match should begin
        end: Optional character offset where the match should end
    """

    article: str
    start: int | None = None
    end: int | None = None


class Match(BaseModel):
    """
    A single match location in text.

    Attributes:
        start: Character offset where the match begins
        end: Character offset where the match ends
        confidence: Match confidence (1.0 for exact, <1.0 for fuzzy)
        article_number: Article number where the match was found (if applicable)
        matched_text: The actual text that was matched
    """

    start: int
    end: int
    confidence: float
    article_number: str | None = None
    matched_text: str = ""


class MatchResult(BaseModel):
    """
    Result of locating a selector in text.

    Use the boolean properties for clean result handling:

        result = selector.locate(text)
        if result.found:
            print(result.match.matched_text)
        elif result.ambiguous:
            print(f"Found {len(result.matches)} matches")
        elif result.orphaned:
            print("Not found")
    """

    status: MatchStatus
    matches: list[Match] = []

    @computed_field  # type: ignore[prop-decorator]
    @property
    def found(self) -> bool:
        """True if exactly one match was found."""
        return self.status == MatchStatus.FOUND

    @computed_field  # type: ignore[prop-decorator]
    @property
    def orphaned(self) -> bool:
        """True if no match was found."""
        return self.status == MatchStatus.ORPHANED

    @computed_field  # type: ignore[prop-decorator]
    @property
    def ambiguous(self) -> bool:
        """True if multiple matches were found."""
        return self.status == MatchStatus.AMBIGUOUS

    @property
    def match(self) -> Match | None:
        """The single match (only valid when found=True)."""
        return self.matches[0] if self.matches else None


class TextQuoteSelector(BaseModel):
    """
    W3C Web Annotation TextQuoteSelector.

    Selects text by specifying an exact quote with optional prefix/suffix context.
    The prefix and suffix help disambiguate when the exact text appears multiple times.

    Example:
        selector = TextQuoteSelector(exact="zorgtoeslag", prefix="op een ")
        result = selector.locate(law)
        if result.found:
            print(result.match.article_number)

    Attributes:
        type: Selector type identifier (always "TextQuoteSelector")
        exact: The exact text to match
        prefix: Optional text that appears before the exact match
        suffix: Optional text that appears after the exact match
    """

    type: str = "TextQuoteSelector"
    exact: str
    prefix: str = ""
    suffix: str = ""
    hint: Hint | None = None

    @classmethod
    def from_annotation(cls, yaml_text: str) -> Self:
        """
        Load a TextQuoteSelector from W3C Web Annotation YAML.

        Parses the target.selector from an annotation body, including
        optional regelrecht:hint for performance optimization.

        Example:
            selector = TextQuoteSelector.from_annotation('''
                target:
                  selector:
                    type: TextQuoteSelector
                    exact: "zorgtoeslag"
                    prefix: "op een "
                    regelrecht:hint:
                      type: CssSelector
                      value: "article[number='2']"
                      refinedBy:
                        type: TextPositionSelector
                        start: 45
                        end: 56
            ''')
        """
        data = yaml.safe_load(yaml_text)
        selector_data = data.get("target", {}).get("selector", {})

        hint = None
        hint_data = selector_data.get("regelrecht:hint")
        if hint_data:
            hint = _parse_hint(hint_data)

        return cls(
            exact=selector_data.get("exact", ""),
            prefix=selector_data.get("prefix", ""),
            suffix=selector_data.get("suffix", ""),
            hint=hint,
        )

    def locate(
        self,
        target: str | Law | list[Article],
        fuzzy_threshold: float = 0.7,
    ) -> MatchResult:
        """
        Locate this selector in text, a law, or articles.

        This implements the resolution algorithm from RFC-004:
        1. If hint is present, try hinted article first
        2. Try exact match (prefix + exact + suffix)
        3. Fall back to fuzzy matching if exact match fails
        4. Return orphaned if no match found above threshold
        5. Return ambiguous if multiple equally-good matches found

        Args:
            target: Text string, Law object, or list of Articles to search
            fuzzy_threshold: Minimum confidence for fuzzy matches (default 0.7)

        Returns:
            MatchResult with status and matches
        """
        from regelrecht.models import Article, Law

        if isinstance(target, str):
            return _locate_in_text(target, self, fuzzy_threshold)
        elif isinstance(target, Law):
            return _locate_in_articles(
                target.articles, self, fuzzy_threshold, self.hint
            )
        elif isinstance(target, list) and all(isinstance(a, Article) for a in target):
            return _locate_in_articles(target, self, fuzzy_threshold, self.hint)
        else:
            msg = f"target must be str, Law, or list[Article], got {type(target)}"
            raise TypeError(msg)


def _parse_hint(hint_data: dict) -> Hint | None:
    """
    Parse a regelrecht:hint from W3C selector format.

    Expected format:
        type: CssSelector
        value: "article[number='2']"
        refinedBy:
          type: TextPositionSelector
          start: 45
          end: 56
    """
    import re

    # Extract article number from CssSelector value
    css_value = hint_data.get("value", "")
    article_match = re.search(r"article\[number=['\"]([^'\"]+)['\"]\]", css_value)
    if not article_match:
        return None

    article = article_match.group(1)

    # Extract position from refinedBy (optional)
    start = None
    end = None
    refined_by = hint_data.get("refinedBy", {})
    if refined_by.get("type") == "TextPositionSelector":
        start = refined_by.get("start")
        end = refined_by.get("end")

    return Hint(article=article, start=start, end=end)


def _locate_in_text(
    text: str,
    selector: TextQuoteSelector,
    fuzzy_threshold: float,
) -> MatchResult:
    """Locate selector in a single text body."""
    # Step 1: Try exact match
    exact_matches = _find_exact_matches(text, selector)
    if exact_matches:
        if len(exact_matches) == 1:
            return MatchResult(status=MatchStatus.FOUND, matches=exact_matches)
        return MatchResult(status=MatchStatus.AMBIGUOUS, matches=exact_matches)

    # Step 2: Try fuzzy matching
    fuzzy_matches = find_fuzzy_matches(text, selector, fuzzy_threshold)
    if fuzzy_matches:
        # Deduplicate overlapping matches - keep the best match for each region
        deduped = _deduplicate_overlapping_matches(fuzzy_matches)
        if len(deduped) == 1:
            return MatchResult(status=MatchStatus.FOUND, matches=deduped)
        # If best match is significantly better than second-best, return it
        if len(deduped) > 1 and deduped[0].confidence - deduped[1].confidence > 0.1:
            return MatchResult(status=MatchStatus.FOUND, matches=[deduped[0]])
        return MatchResult(status=MatchStatus.AMBIGUOUS, matches=deduped)

    # Step 3: No match found
    return MatchResult(status=MatchStatus.ORPHANED, matches=[])


def _locate_in_articles(
    articles: list[Article],
    selector: TextQuoteSelector,
    fuzzy_threshold: float,
    hint: Hint | None = None,
) -> MatchResult:
    """
    Locate selector across multiple articles.

    If a hint is provided, tries the hinted article first. If found there,
    returns immediately. If not found, falls back to searching all articles.
    """
    # Step 0: If hint provided, try hinted article first
    if hint:
        result = _try_hint(articles, selector, fuzzy_threshold, hint)
        if result.found:
            return result
        # Hint failed, fall through to full search

    # Step 1: Try exact match across all articles
    all_matches: list[Match] = []

    for article in articles:
        exact_matches = _find_exact_matches(article.text, selector)
        for match in exact_matches:
            match.article_number = article.number
            all_matches.append(match)

    if all_matches:
        if len(all_matches) == 1:
            return MatchResult(status=MatchStatus.FOUND, matches=all_matches)
        return MatchResult(status=MatchStatus.AMBIGUOUS, matches=all_matches)

    # Step 2: Try fuzzy matching across all articles
    for article in articles:
        fuzzy_matches = find_fuzzy_matches(article.text, selector, fuzzy_threshold)
        for match in fuzzy_matches:
            match.article_number = article.number
            all_matches.append(match)

    if all_matches:
        deduped = _deduplicate_overlapping_matches(all_matches)
        if len(deduped) == 1:
            return MatchResult(status=MatchStatus.FOUND, matches=deduped)
        if len(deduped) > 1 and deduped[0].confidence - deduped[1].confidence > 0.1:
            return MatchResult(status=MatchStatus.FOUND, matches=[deduped[0]])
        return MatchResult(status=MatchStatus.AMBIGUOUS, matches=deduped)

    return MatchResult(status=MatchStatus.ORPHANED, matches=[])


def _try_hint(
    articles: list[Article],
    selector: TextQuoteSelector,
    fuzzy_threshold: float,
    hint: Hint,
) -> MatchResult:
    """
    Try to find the selector at the hinted location.

    If hint has position (start/end), checks that specific range first.
    Otherwise searches the entire hinted article.
    """
    # Find the hinted article
    hinted_article = None
    for article in articles:
        if article.number == hint.article:
            hinted_article = article
            break

    if not hinted_article:
        # Hinted article not found, return orphaned to trigger fallback
        return MatchResult(status=MatchStatus.ORPHANED, matches=[])

    # If hint has position, check that specific range first
    if hint.start is not None and hint.end is not None:
        text = hinted_article.text
        if hint.start < len(text) and hint.end <= len(text):
            hinted_text = text[hint.start : hint.end]
            # Check if the exact text matches at the hinted position
            if hinted_text == selector.exact:
                # Verify prefix/suffix in context
                match = _verify_match_at_position(
                    text, selector, hint.start, hint.end, hinted_article.number
                )
                if match:
                    return MatchResult(status=MatchStatus.FOUND, matches=[match])

    # Position hint failed or wasn't provided - search entire hinted article
    exact_matches = _find_exact_matches(hinted_article.text, selector)
    for match in exact_matches:
        match.article_number = hinted_article.number

    if exact_matches:
        if len(exact_matches) == 1:
            return MatchResult(status=MatchStatus.FOUND, matches=exact_matches)
        return MatchResult(status=MatchStatus.AMBIGUOUS, matches=exact_matches)

    # Try fuzzy in hinted article
    fuzzy_matches = find_fuzzy_matches(hinted_article.text, selector, fuzzy_threshold)
    for match in fuzzy_matches:
        match.article_number = hinted_article.number

    if fuzzy_matches:
        deduped = _deduplicate_overlapping_matches(fuzzy_matches)
        if len(deduped) == 1:
            return MatchResult(status=MatchStatus.FOUND, matches=deduped)
        if len(deduped) > 1 and deduped[0].confidence - deduped[1].confidence > 0.1:
            return MatchResult(status=MatchStatus.FOUND, matches=[deduped[0]])

    # Not found in hinted article
    return MatchResult(status=MatchStatus.ORPHANED, matches=[])


def _verify_match_at_position(
    text: str,
    selector: TextQuoteSelector,
    start: int,
    end: int,
    article_number: str,
) -> Match | None:
    """Verify that a match at a specific position has correct prefix/suffix."""
    # Check prefix
    if selector.prefix:
        prefix_start = max(0, start - len(selector.prefix) - 1)
        actual_prefix = text[prefix_start:start]
        if selector.prefix.strip() not in actual_prefix.strip():
            return None

    # Check suffix
    if selector.suffix:
        suffix_end = min(len(text), end + len(selector.suffix) + 1)
        actual_suffix = text[end:suffix_end]
        if selector.suffix.strip() not in actual_suffix.strip():
            return None

    return Match(
        start=start,
        end=end,
        confidence=1.0,
        article_number=article_number,
        matched_text=text[start:end],
    )


def _deduplicate_overlapping_matches(matches: list[Match]) -> list[Match]:
    """Remove overlapping matches, keeping only the highest-confidence one."""
    if not matches:
        return []

    # Sort by confidence (highest first)
    sorted_matches = sorted(matches, key=lambda m: m.confidence, reverse=True)

    result: list[Match] = []
    for match in sorted_matches:
        # Check if this match overlaps with any already-selected match
        overlaps = False
        for selected in result:
            # Two ranges overlap if: start1 < end2 AND start2 < end1
            if match.start < selected.end and selected.start < match.end:
                overlaps = True
                break
        if not overlaps:
            result.append(match)

    return result


def _find_exact_matches(text: str, selector: TextQuoteSelector) -> list[Match]:
    """
    Find all exact matches of the selector in text.

    An exact match requires:
    - The exact text is found
    - If prefix is provided, it must appear immediately before (whitespace-normalized)
    - If suffix is provided, it must appear immediately after (whitespace-normalized)
    """
    matches: list[Match] = []
    search_start = 0

    while True:
        # Find next occurrence of exact text
        pos = text.find(selector.exact, search_start)
        if pos == -1:
            break

        # Check prefix (with whitespace normalization)
        if selector.prefix:
            # Look back for the prefix, allowing for whitespace flexibility
            prefix_stripped = selector.prefix.strip()
            prefix_start = pos - len(selector.prefix) - 1  # Allow one extra char
            prefix_start = max(0, prefix_start)
            actual_prefix = text[prefix_start:pos]
            # Check if the stripped prefix appears in the actual prefix
            if prefix_stripped not in actual_prefix.strip():
                search_start = pos + 1
                continue

        # Check suffix (with whitespace normalization)
        exact_end = pos + len(selector.exact)
        if selector.suffix:
            suffix_stripped = selector.suffix.strip()
            suffix_end = exact_end + len(selector.suffix) + 1  # Allow one extra char
            suffix_end = min(len(text), suffix_end)
            actual_suffix = text[exact_end:suffix_end]
            # Check if the stripped suffix appears in the actual suffix
            if suffix_stripped not in actual_suffix.strip():
                search_start = pos + 1
                continue

        # Found a match
        matches.append(
            Match(
                start=pos,
                end=exact_end,
                confidence=1.0,
                matched_text=selector.exact,
            )
        )
        search_start = pos + 1

    return matches

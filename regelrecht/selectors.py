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

    @classmethod
    def from_annotation(cls, yaml_text: str) -> Self:
        """
        Load a TextQuoteSelector from W3C Web Annotation YAML.

        Parses the target.selector from an annotation body.

        Example:
            selector = TextQuoteSelector.from_annotation('''
                target:
                  selector:
                    type: TextQuoteSelector
                    exact: "zorgtoeslag"
                    prefix: "op een "
            ''')
        """
        data = yaml.safe_load(yaml_text)
        selector_data = data.get("target", {}).get("selector", {})
        return cls(
            exact=selector_data.get("exact", ""),
            prefix=selector_data.get("prefix", ""),
            suffix=selector_data.get("suffix", ""),
        )

    def locate(
        self,
        target: str | Law | list[Article],
        fuzzy_threshold: float = 0.7,
    ) -> MatchResult:
        """
        Locate this selector in text, a law, or articles.

        This implements the resolution algorithm from RFC-004:
        1. Try exact match first (prefix + exact + suffix)
        2. Fall back to fuzzy matching if exact match fails
        3. Return orphaned if no match found above threshold
        4. Return ambiguous if multiple equally-good matches found

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
            return _locate_in_articles(target.articles, self, fuzzy_threshold)
        elif isinstance(target, list) and all(isinstance(a, Article) for a in target):
            return _locate_in_articles(target, self, fuzzy_threshold)
        else:
            msg = f"target must be str, Law, or list[Article], got {type(target)}"
            raise TypeError(msg)


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
) -> MatchResult:
    """Locate selector across multiple articles."""
    all_matches: list[Match] = []

    for article in articles:
        # Try exact match in this article
        exact_matches = _find_exact_matches(article.text, selector)
        for match in exact_matches:
            match.article_number = article.number
            all_matches.append(match)

    # If we found exact matches, return them
    if all_matches:
        if len(all_matches) == 1:
            return MatchResult(status=MatchStatus.FOUND, matches=all_matches)
        return MatchResult(status=MatchStatus.AMBIGUOUS, matches=all_matches)

    # Try fuzzy matching across articles
    for article in articles:
        fuzzy_matches = find_fuzzy_matches(article.text, selector, fuzzy_threshold)
        for match in fuzzy_matches:
            match.article_number = article.number
            all_matches.append(match)

    if all_matches:
        # Deduplicate overlapping matches within each article
        deduped = _deduplicate_overlapping_matches(all_matches)
        if len(deduped) == 1:
            return MatchResult(status=MatchStatus.FOUND, matches=deduped)
        # If best match is significantly better than second-best, return it
        if len(deduped) > 1 and deduped[0].confidence - deduped[1].confidence > 0.1:
            return MatchResult(status=MatchStatus.FOUND, matches=[deduped[0]])
        return MatchResult(status=MatchStatus.AMBIGUOUS, matches=deduped)

    return MatchResult(status=MatchStatus.ORPHANED, matches=[])


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

"""
Fuzzy matching implementation for TextQuoteSelector.

Uses difflib.SequenceMatcher for similarity scoring as specified in RFC-004.
"""

from difflib import SequenceMatcher

from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from annotation.selector import Match, TextQuoteSelector


def similarity_score(s1: str, s2: str) -> float:
    """
    Calculate similarity score between two strings using SequenceMatcher.

    Returns a float between 0.0 (completely different) and 1.0 (identical).
    """
    if not s1 and not s2:
        return 1.0
    if not s1 or not s2:
        return 0.0
    return SequenceMatcher(None, s1, s2).ratio()


def find_fuzzy_matches(
    text: str,
    selector: "TextQuoteSelector",
    threshold: float = 0.7,
) -> list["Match"]:
    """
    Find fuzzy matches of the selector in text.

    Algorithm from RFC-004:
    1. Find candidate positions where similar text might be
    2. For each candidate, calculate weighted score:
       - exact_score * 0.5 + prefix_score * 0.25 + suffix_score * 0.25
    3. Return matches above threshold

    Args:
        text: The text to search in
        selector: The TextQuoteSelector to match
        threshold: Minimum weighted score for a match (default 0.7)

    Returns:
        List of Match objects sorted by confidence (highest first)
    """
    from annotation.selector import Match

    matches: list[Match] = []

    # Generate candidate positions by finding similar substrings
    candidates = _find_candidates(text, selector.exact)

    for start, end, candidate_text in candidates:
        # Get context around the candidate
        prefix_len = len(selector.prefix) if selector.prefix else 0
        suffix_len = len(selector.suffix) if selector.suffix else 0

        prefix_start = max(0, start - prefix_len)
        actual_prefix = text[prefix_start:start]

        suffix_end = min(len(text), end + suffix_len)
        actual_suffix = text[end:suffix_end]

        # Calculate similarity scores
        exact_score = similarity_score(selector.exact, candidate_text)

        if selector.prefix:
            prefix_score = similarity_score(selector.prefix, actual_prefix)
        else:
            prefix_score = 1.0  # No prefix to match = perfect match

        if selector.suffix:
            suffix_score = similarity_score(selector.suffix, actual_suffix)
        else:
            suffix_score = 1.0  # No suffix to match = perfect match

        # Weighted score: exact counts more than context
        weighted_score = (
            (exact_score * 0.5) + (prefix_score * 0.25) + (suffix_score * 0.25)
        )

        if weighted_score >= threshold:
            matches.append(
                Match(
                    start=start,
                    end=end,
                    confidence=weighted_score,
                    matched_text=candidate_text,
                )
            )

    # Sort by confidence (highest first)
    matches.sort(key=lambda m: m.confidence, reverse=True)

    return matches


def _find_candidates(text: str, exact: str) -> list[tuple[int, int, str]]:
    """
    Find candidate positions in text that might match the exact string.

    This is a simple approach that finds:
    1. Exact occurrences
    2. Substrings of similar length that share words with the exact string

    Returns list of (start, end, candidate_text) tuples.
    """
    candidates: list[tuple[int, int, str]] = []

    # First, find exact occurrences
    search_start = 0
    while True:
        pos = text.find(exact, search_start)
        if pos == -1:
            break
        candidates.append((pos, pos + len(exact), exact))
        search_start = pos + 1

    # Then, find similar-length substrings that might be fuzzy matches
    # Use a sliding window approach
    exact_len = len(exact)
    window_tolerance = int(exact_len * 0.3)  # Allow 30% length variation

    for window_size in range(
        max(1, exact_len - window_tolerance), exact_len + window_tolerance + 1
    ):
        for i in range(len(text) - window_size + 1):
            candidate = text[i : i + window_size]

            # Skip if we already have this exact position
            if any(s == i and e == i + window_size for s, e, _ in candidates):
                continue

            # Quick check: do they share any significant words?
            if _shares_significant_content(exact, candidate):
                candidates.append((i, i + window_size, candidate))

    return candidates


def _shares_significant_content(s1: str, s2: str) -> bool:
    """
    Check if two strings share significant content.

    This is a quick filter to avoid expensive similarity calculations
    on completely unrelated strings.
    """
    # Extract words (simple tokenization)
    words1 = set(s1.lower().split())
    words2 = set(s2.lower().split())

    # Check for word overlap
    common = words1 & words2

    # Need at least one significant word in common
    # (words longer than 3 characters to avoid matching on articles/prepositions)
    significant_common = [w for w in common if len(w) > 3]

    return len(significant_common) > 0

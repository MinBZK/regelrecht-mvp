"""
Step definitions for TextQuoteSelector annotation resolution tests.

These tests verify RFC-004: Stand-off Annotations for Legal Texts.
"""

from behave import given, then, when  # type: ignore[import-untyped]

from regelrecht.models import Law
from regelrecht.selectors import MatchResult, MatchStatus, TextQuoteSelector


# === Law Setup ===


@given('law version "{version}":')  # type: ignore[misc]
def step_given_law_version(context, version):
    """Parse law from YAML format."""
    _store_law_version(context, version, context.text)


@given('law version "{version}" with "verzekerde" appearing 79 times')  # type: ignore[misc]
def step_given_law_ambiguous(context, version):
    """Create law with verzekerde appearing many times (simulated)."""
    if not hasattr(context, "articles"):
        context.articles = {}

    # Create text with "verzekerde" appearing 79 times
    base_text = "De verzekerde heeft aanspraak. "
    context.full_text = base_text * 79
    context.current_version = version


def _store_law_version(context, version, yaml_text):
    """Store a law version from YAML text."""
    if not hasattr(context, "laws"):
        context.laws = {}

    context.laws[version] = Law.from_yaml(yaml_text)
    context.current_version = version


# === Annotation Setup ===


@given("annotation:")  # type: ignore[misc]
def step_given_annotation(context):
    """Parse annotation from YAML format."""
    context.selector = TextQuoteSelector.from_annotation(context.text)


@given('annotation created on version "{version}" targeting article "{article}":')  # type: ignore[misc]
def step_given_annotation_versioned(context, version, article):
    """Parse annotation with version info."""
    context.selector = TextQuoteSelector.from_annotation(context.text)
    context.annotation_source_version = version
    context.annotation_target_article = article


@given('annotation created on version "{version}":')  # type: ignore[misc]
def step_given_annotation_on_version(context, version):
    """Parse annotation on specific version."""
    context.selector = TextQuoteSelector.from_annotation(context.text)
    context.annotation_source_version = version


@given("annotation with insufficient context:")  # type: ignore[misc]
def step_given_annotation_insufficient(context):
    """Parse annotation with minimal context (likely ambiguous)."""
    context.selector = TextQuoteSelector.from_annotation(context.text)


# === Resolution Actions ===


@when("I resolve the annotation")  # type: ignore[misc]
def step_when_resolve(context):
    """Resolve annotation against current version using selector.locate()."""
    if hasattr(context, "full_text"):
        context.result = context.selector.locate(context.full_text)
    elif hasattr(context, "laws") and context.current_version in context.laws:
        law = context.laws[context.current_version]
        context.result = context.selector.locate(law)
    else:
        context.result = MatchResult(status=MatchStatus.ORPHANED, matches=[])


@when('I resolve the annotation against version "{version}"')  # type: ignore[misc]
def step_when_resolve_version(context, version):
    """Resolve annotation against specific version using selector.locate()."""
    if hasattr(context, "laws") and version in context.laws:
        law = context.laws[version]
        context.result = context.selector.locate(law)
    else:
        context.result = MatchResult(status=MatchStatus.ORPHANED, matches=[])


# === Assertions ===


@then("the result is FOUND with confidence {confidence:f}")  # type: ignore[misc]
def step_then_found_confidence(context, confidence):
    """Assert match was found with expected confidence."""
    assert context.result.found, f"Expected found but got {context.result.status}"
    assert len(context.result.matches) == 1, (
        f"Expected 1 match but got {len(context.result.matches)}"
    )
    actual = context.result.match.confidence
    assert abs(actual - confidence) < 0.01, (
        f"Expected confidence {confidence} but got {actual}"
    )


@then("the result is FOUND with confidence above {threshold:f}")  # type: ignore[misc]
def step_then_found_above(context, threshold):
    """Assert match was found with confidence above threshold."""
    assert context.result.found, f"Expected found but got {context.result.status}"
    assert len(context.result.matches) >= 1, "Expected at least 1 match"
    actual = context.result.match.confidence
    assert actual > threshold, f"Expected confidence above {threshold} but got {actual}"


@then('the match is in article "{article_num}"')  # type: ignore[misc]
def step_then_match_in_article(context, article_num):
    """Assert match was found in expected article."""
    assert len(context.result.matches) >= 1, "No matches found"
    actual = context.result.match.article_number
    assert actual == article_num, f"Expected article {article_num} but got {actual}"


@then('the matched text contains "{expected_text}"')  # type: ignore[misc]
def step_then_matched_text_contains(context, expected_text):
    """Assert matched text contains expected substring."""
    assert len(context.result.matches) >= 1, "No matches found"
    actual = context.result.match.matched_text
    assert expected_text in actual, (
        f"Expected '{expected_text}' in matched text but got '{actual}'"
    )


@then("the result is ORPHANED")  # type: ignore[misc]
def step_then_orphaned(context):
    """Assert annotation could not be resolved."""
    assert context.result.orphaned, f"Expected orphaned but got {context.result.status}"


@then("no match is found")  # type: ignore[misc]
def step_then_no_match(context):
    """Assert no matches were found."""
    assert len(context.result.matches) == 0, (
        f"Expected no matches but got {len(context.result.matches)}"
    )


@then("the result is AMBIGUOUS")  # type: ignore[misc]
def step_then_ambiguous(context):
    """Assert annotation has multiple matches."""
    assert context.result.ambiguous, (
        f"Expected ambiguous but got {context.result.status}"
    )


@then("multiple matches are found")  # type: ignore[misc]
def step_then_multiple_matches(context):
    """Assert multiple matches were found."""
    assert len(context.result.matches) > 1, (
        f"Expected multiple matches but got {len(context.result.matches)}"
    )


@then("exactly {count:d} match is found")  # type: ignore[misc]
def step_then_exact_matches(context, count):
    """Assert exact number of matches."""
    assert len(context.result.matches) == count, (
        f"Expected {count} matches but got {len(context.result.matches)}"
    )

"""
Annotation resolution module for TextQuoteSelector-based annotations.

This module implements the W3C Web Annotation TextQuoteSelector resolution
algorithm as specified in RFC-004.
"""

from annotation.selector import Match, MatchResult, TextQuoteSelector, resolve_selector

__all__ = ["TextQuoteSelector", "Match", "MatchResult", "resolve_selector"]

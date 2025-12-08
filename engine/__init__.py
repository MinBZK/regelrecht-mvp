"""
Regelrecht Engine - Article-based law execution engine

This package implements an execution engine for article-based legal specifications
using the regelrecht:// URI reference format.
"""

from engine.article_loader import Article, ArticleBasedLaw
from engine.uri_resolver import RegelrechtURI
from engine.service import LawExecutionService
from engine.context import NoLegalBasisError

__all__ = [
    "Article",
    "ArticleBasedLaw",
    "RegelrechtURI",
    "LawExecutionService",
    "NoLegalBasisError",
]

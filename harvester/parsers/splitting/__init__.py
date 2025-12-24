"""Splitting module for declarative article splitting.

This module provides a registry-based approach to splitting Dutch law
articles into components, replacing hardcoded tag checks with declarative
hierarchy specifications.
"""

from harvester.parsers.splitting.config import create_dutch_law_hierarchy
from harvester.parsers.splitting.engine import SplitEngine
from harvester.parsers.splitting.protocols import (
    ElementSpec,
    SplitContext,
    SplitStrategy,
)
from harvester.parsers.splitting.registry import HierarchyRegistry
from harvester.parsers.splitting.strategies import (
    DepthLimitedStrategy,
    LeafSplitStrategy,
)

__all__ = [
    "ElementSpec",
    "SplitContext",
    "SplitStrategy",
    "HierarchyRegistry",
    "SplitEngine",
    "LeafSplitStrategy",
    "DepthLimitedStrategy",
    "create_dutch_law_hierarchy",
]

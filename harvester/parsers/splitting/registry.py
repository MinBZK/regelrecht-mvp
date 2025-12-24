"""Registry for element hierarchy specifications."""

from __future__ import annotations

from harvester.parsers.splitting.protocols import ElementSpec


class HierarchyRegistry:
    """Registry for element hierarchy specifications.

    Maps XML tag names to their ElementSpec definitions, providing
    a declarative way to describe the document structure.
    """

    def __init__(self) -> None:
        """Initialize an empty registry."""
        self._specs: dict[str, ElementSpec] = {}

    def register(self, spec: ElementSpec) -> None:
        """Register an element specification.

        Args:
            spec: The element specification to register
        """
        self._specs[spec.tag] = spec

    def get_spec(self, tag: str) -> ElementSpec | None:
        """Get specification for a tag.

        Args:
            tag: The tag name (without namespace)

        Returns:
            The element specification, or None if not registered
        """
        return self._specs.get(tag)

    def is_structural(self, tag: str) -> bool:
        """Check if tag is a registered structural element.

        Args:
            tag: The tag name

        Returns:
            True if the tag has a registered specification
        """
        return tag in self._specs

    def registered_tags(self) -> set[str]:
        """Return set of all registered tag names."""
        return set(self._specs.keys())

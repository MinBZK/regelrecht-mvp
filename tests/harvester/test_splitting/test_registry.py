"""Tests for HierarchyRegistry."""

from harvester.parsers.splitting import ElementSpec, HierarchyRegistry


class TestHierarchyRegistry:
    """Tests for HierarchyRegistry class."""

    def test_register_and_get_spec(self) -> None:
        """Register a spec and retrieve it."""
        registry = HierarchyRegistry()
        spec = ElementSpec(tag="artikel", children=["lid"], is_split_point=True)

        registry.register(spec)
        result = registry.get_spec("artikel")

        assert result is spec

    def test_get_spec_returns_none_for_unregistered(self) -> None:
        """Return None for unregistered tags."""
        registry = HierarchyRegistry()

        result = registry.get_spec("unknown")

        assert result is None

    def test_is_structural(self) -> None:
        """Check if tag is structural."""
        registry = HierarchyRegistry()
        registry.register(ElementSpec(tag="artikel", is_split_point=True))

        assert registry.is_structural("artikel") is True
        assert registry.is_structural("unknown") is False

    def test_registered_tags(self) -> None:
        """Return set of registered tags."""
        registry = HierarchyRegistry()
        registry.register(ElementSpec(tag="artikel", is_split_point=True))
        registry.register(ElementSpec(tag="lid", is_split_point=True))

        tags = registry.registered_tags()

        assert tags == {"artikel", "lid"}


class TestElementSpec:
    """Tests for ElementSpec dataclass."""

    def test_default_values(self) -> None:
        """ElementSpec has sensible defaults."""
        spec = ElementSpec(tag="test")

        assert spec.tag == "test"
        assert spec.children == []
        assert spec.number_source is None
        assert spec.content_tags == []
        assert spec.is_split_point is False
        assert spec.skip_for_number == []

    def test_full_spec(self) -> None:
        """ElementSpec with all values set."""
        spec = ElementSpec(
            tag="lid",
            children=["lijst", "al"],
            number_source="lidnr",
            content_tags=["al"],
            is_split_point=True,
            skip_for_number=["lidnr"],
        )

        assert spec.tag == "lid"
        assert spec.children == ["lijst", "al"]
        assert spec.number_source == "lidnr"
        assert spec.content_tags == ["al"]
        assert spec.is_split_point is True
        assert spec.skip_for_number == ["lidnr"]

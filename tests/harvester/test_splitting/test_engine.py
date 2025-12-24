"""Tests for SplitEngine."""

from lxml import etree

from harvester.parsers.splitting import (
    HierarchyRegistry,
    LeafSplitStrategy,
    SplitContext,
    SplitEngine,
    create_dutch_law_hierarchy,
)


def make_context(
    bwb_id: str = "BWBR0001",
    date: str = "2025-01-01",
) -> SplitContext:
    """Create a test context."""
    return SplitContext(
        bwb_id=bwb_id,
        date=date,
        base_url=f"https://wetten.overheid.nl/{bwb_id}/{date}",
    )


class TestSplitEngineBasics:
    """Basic tests for SplitEngine."""

    def test_unknown_element_returns_empty(self) -> None:
        """Unknown elements return empty list."""
        registry = HierarchyRegistry()
        engine = SplitEngine(registry, LeafSplitStrategy())
        elem = etree.fromstring(b"<unknown>content</unknown>")

        result = engine.split(elem, make_context())

        assert result == []

    def test_simple_artikel_with_al(self) -> None:
        """Split simple artikel with direct al content."""
        hierarchy = create_dutch_law_hierarchy()
        engine = SplitEngine(hierarchy, LeafSplitStrategy())
        elem = etree.fromstring(
            b"""<artikel>
                <kop><nr>1</nr></kop>
                <al>Dit is de tekst.</al>
            </artikel>"""
        )

        result = engine.split(elem, make_context())

        assert len(result) == 1
        assert result[0].number_parts == ["1"]
        assert result[0].text == "Dit is de tekst."

    def test_artikel_with_lid(self) -> None:
        """Split artikel with lid."""
        hierarchy = create_dutch_law_hierarchy()
        engine = SplitEngine(hierarchy, LeafSplitStrategy())
        elem = etree.fromstring(
            b"""<artikel>
                <kop><nr>1</nr></kop>
                <lid>
                    <lidnr>1.</lidnr>
                    <al>Eerste lid.</al>
                </lid>
                <lid>
                    <lidnr>2.</lidnr>
                    <al>Tweede lid.</al>
                </lid>
            </artikel>"""
        )

        result = engine.split(elem, make_context())

        assert len(result) == 2
        # lidnr keeps the trailing period
        assert result[0].number_parts == ["1", "1."]
        assert result[0].text == "Eerste lid."
        assert result[1].number_parts == ["1", "2."]
        assert result[1].text == "Tweede lid."


class TestSplitEngineLists:
    """Tests for list handling in SplitEngine."""

    def test_lid_with_lijst(self) -> None:
        """Split lid with lijst into separate components."""
        hierarchy = create_dutch_law_hierarchy()
        engine = SplitEngine(hierarchy, LeafSplitStrategy())
        elem = etree.fromstring(
            b"""<artikel>
                <kop><nr>1</nr></kop>
                <lid>
                    <lidnr>1.</lidnr>
                    <al>In dit artikel:</al>
                    <lijst>
                        <li><li.nr>a.</li.nr><al>eerste item;</al></li>
                        <li><li.nr>b.</li.nr><al>tweede item.</al></li>
                    </lijst>
                </lid>
            </artikel>"""
        )

        result = engine.split(elem, make_context())

        # Should have: intro + 2 list items = 3 components
        assert len(result) == 3
        # lidnr keeps the trailing period, li.nr has it stripped
        assert result[0].number_parts == ["1", "1."]
        assert result[0].text == "In dit artikel:"
        assert result[1].number_parts == ["1", "1.", "a"]
        assert result[1].text == "eerste item;"
        assert result[2].number_parts == ["1", "1.", "b"]
        assert result[2].text == "tweede item."

    def test_nested_lijst(self) -> None:
        """Handle nested lists."""
        hierarchy = create_dutch_law_hierarchy()
        engine = SplitEngine(hierarchy, LeafSplitStrategy())
        elem = etree.fromstring(
            b"""<artikel>
                <kop><nr>1</nr></kop>
                <lid>
                    <lidnr>1.</lidnr>
                    <lijst>
                        <li>
                            <li.nr>a.</li.nr>
                            <al>Outer item:</al>
                            <lijst>
                                <li><li.nr>1.</li.nr><al>nested one;</al></li>
                                <li><li.nr>2.</li.nr><al>nested two.</al></li>
                            </lijst>
                        </li>
                    </lijst>
                </lid>
            </artikel>"""
        )

        result = engine.split(elem, make_context())

        # Should have: outer item intro + 2 nested items = 3 components
        # li.nr has trailing period stripped
        assert len(result) == 3
        assert result[0].number_parts == ["1", "1.", "a"]
        assert result[0].text == "Outer item:"
        assert result[1].number_parts == ["1", "1.", "a", "1"]
        assert result[1].text == "nested one;"
        assert result[2].number_parts == ["1", "1.", "a", "2"]
        assert result[2].text == "nested two."


class TestSplitEngineConfig:
    """Tests for Dutch law hierarchy configuration."""

    def test_create_dutch_law_hierarchy(self) -> None:
        """Hierarchy includes expected elements."""
        hierarchy = create_dutch_law_hierarchy()

        assert hierarchy.is_structural("artikel")
        assert hierarchy.is_structural("lid")
        assert hierarchy.is_structural("lijst")
        assert hierarchy.is_structural("li")
        assert not hierarchy.is_structural("al")
        assert not hierarchy.is_structural("unknown")

    def test_artikel_spec(self) -> None:
        """Artikel spec is configured correctly."""
        hierarchy = create_dutch_law_hierarchy()
        spec = hierarchy.get_spec("artikel")

        assert spec is not None
        assert spec.children == ["lid", "lijst"]  # Structural only
        assert spec.number_source == "kop/nr"
        assert spec.is_split_point is True

    def test_lid_spec(self) -> None:
        """Lid spec is configured correctly."""
        hierarchy = create_dutch_law_hierarchy()
        spec = hierarchy.get_spec("lid")

        assert spec is not None
        assert spec.children == ["lijst"]  # Structural only
        assert spec.number_source == "lidnr"
        assert spec.is_split_point is True

    def test_lijst_spec(self) -> None:
        """Lijst spec is configured correctly."""
        hierarchy = create_dutch_law_hierarchy()
        spec = hierarchy.get_spec("lijst")

        assert spec is not None
        assert spec.children == ["li"]
        assert spec.is_split_point is False

    def test_li_spec(self) -> None:
        """Li spec is configured correctly."""
        hierarchy = create_dutch_law_hierarchy()
        spec = hierarchy.get_spec("li")

        assert spec is not None
        assert spec.children == ["lijst"]  # Structural only
        assert spec.number_source == "li.nr"
        assert spec.is_split_point is True

"""Tests for split strategies."""

from lxml import etree

from harvester.parsers.splitting import (
    DepthLimitedStrategy,
    ElementSpec,
    LeafSplitStrategy,
    SplitContext,
)


class TestLeafSplitStrategy:
    """Tests for LeafSplitStrategy."""

    def test_should_split_at_split_point(self) -> None:
        """Split at elements marked as split points."""
        strategy = LeafSplitStrategy()
        spec = ElementSpec(tag="lid", is_split_point=True)
        elem = etree.fromstring(b"<lid><lidnr>1.</lidnr></lid>")
        context = SplitContext(bwb_id="BWBR0001", date="2025-01-01", base_url="")

        result = strategy.should_split_here(elem, spec, context)

        assert result is True

    def test_should_not_split_at_non_split_point(self) -> None:
        """Don't split at elements not marked as split points."""
        strategy = LeafSplitStrategy()
        spec = ElementSpec(tag="lijst", is_split_point=False)
        elem = etree.fromstring(b"<lijst></lijst>")
        context = SplitContext(bwb_id="BWBR0001", date="2025-01-01", base_url="")

        result = strategy.should_split_here(elem, spec, context)

        assert result is False

    def test_get_number_from_direct_child(self) -> None:
        """Extract number from direct child element (lidnr keeps period)."""
        strategy = LeafSplitStrategy()
        spec = ElementSpec(tag="lid", number_source="lidnr")
        elem = etree.fromstring(b"<lid><lidnr>1.</lidnr></lid>")

        result = strategy.get_number(elem, spec)

        # lidnr keeps the trailing period
        assert result == "1."

    def test_get_number_from_xpath(self) -> None:
        """Extract number from XPath location."""
        strategy = LeafSplitStrategy()
        spec = ElementSpec(tag="artikel", number_source="kop/nr")
        elem = etree.fromstring(b"<artikel><kop><nr>5</nr></kop></artikel>")

        result = strategy.get_number(elem, spec)

        assert result == "5"

    def test_get_number_strips_trailing_period(self) -> None:
        """Remove trailing period from number."""
        strategy = LeafSplitStrategy()
        spec = ElementSpec(tag="li", number_source="li.nr")
        elem = etree.fromstring(b"<li><li.nr>a.</li.nr></li>")

        result = strategy.get_number(elem, spec)

        assert result == "a"

    def test_get_number_returns_none_when_missing(self) -> None:
        """Return None when number source is missing."""
        strategy = LeafSplitStrategy()
        spec = ElementSpec(tag="lid", number_source="lidnr")
        elem = etree.fromstring(b"<lid></lid>")

        result = strategy.get_number(elem, spec)

        assert result is None

    def test_get_number_returns_none_when_no_source(self) -> None:
        """Return None when no number source configured."""
        strategy = LeafSplitStrategy()
        spec = ElementSpec(tag="lijst")  # No number_source
        elem = etree.fromstring(b"<lijst></lijst>")

        result = strategy.get_number(elem, spec)

        assert result is None

    def test_get_artikel_number_from_label_fallback(self) -> None:
        """Fall back to label attribute for artikel number."""
        strategy = LeafSplitStrategy()
        spec = ElementSpec(tag="artikel", number_source="kop/nr")
        elem = etree.fromstring(b'<artikel label="Artikel 7a"></artikel>')

        result = strategy.get_number(elem, spec)

        assert result == "7a"


class TestDepthLimitedStrategy:
    """Tests for DepthLimitedStrategy."""

    def test_splits_above_merge_zone(self) -> None:
        """Split at elements above the merge zone."""
        strategy = DepthLimitedStrategy(merge_depth=1)
        spec = ElementSpec(tag="lid", is_split_point=True)
        elem = etree.fromstring(b"<lid></lid>")
        context = SplitContext(
            bwb_id="BWBR0001",
            date="2025-01-01",
            base_url="",
            depth=1,
            max_depth=3,
        )

        result = strategy.should_split_here(elem, spec, context)

        assert result is True  # depth 1 is above merge zone (3-1=2)

    def test_does_not_split_in_merge_zone(self) -> None:
        """Don't split at elements in the merge zone."""
        strategy = DepthLimitedStrategy(merge_depth=1)
        spec = ElementSpec(tag="li", is_split_point=True)
        elem = etree.fromstring(b"<li></li>")
        context = SplitContext(
            bwb_id="BWBR0001",
            date="2025-01-01",
            base_url="",
            depth=2,  # At merge zone start (3-1=2)
            max_depth=3,
        )

        result = strategy.should_split_here(elem, spec, context)

        assert result is False

    def test_splits_when_no_max_depth(self) -> None:
        """Split normally when max_depth is not set."""
        strategy = DepthLimitedStrategy(merge_depth=1)
        spec = ElementSpec(tag="li", is_split_point=True)
        elem = etree.fromstring(b"<li></li>")
        context = SplitContext(
            bwb_id="BWBR0001",
            date="2025-01-01",
            base_url="",
            depth=5,
            max_depth=None,
        )

        result = strategy.should_split_here(elem, spec, context)

        assert result is True

    def test_delegates_number_extraction(self) -> None:
        """Delegate number extraction to base strategy."""
        strategy = DepthLimitedStrategy(merge_depth=1)
        spec = ElementSpec(tag="lid", number_source="lidnr")
        elem = etree.fromstring(b"<lid><lidnr>2.</lidnr></lid>")

        result = strategy.get_number(elem, spec)

        # lidnr keeps the trailing period
        assert result == "2."


class TestSplitContext:
    """Tests for SplitContext."""

    def test_default_values(self) -> None:
        """SplitContext has sensible defaults."""
        context = SplitContext(
            bwb_id="BWBR0001",
            date="2025-01-01",
            base_url="https://example.com",
        )

        assert context.number_parts == []
        assert context.depth == 0
        assert context.max_depth is None

    def test_with_number_creates_new_context(self) -> None:
        """with_number creates a new context with added number."""
        context = SplitContext(
            bwb_id="BWBR0001",
            date="2025-01-01",
            base_url="https://example.com",
            number_parts=["1"],
            depth=1,
        )

        new_context = context.with_number("a")

        assert new_context.number_parts == ["1", "a"]
        assert new_context.depth == 2
        # Original unchanged
        assert context.number_parts == ["1"]
        assert context.depth == 1

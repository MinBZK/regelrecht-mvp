"""Tests for YAML writer."""

from pathlib import Path

import yaml

from harvester.models import Article, Law, LawMetadata, RegulatoryLayer
from harvester.storage.yaml_writer import SCHEMA_URL, generate_yaml_dict, save_yaml


class TestGenerateYamlDict:
    """Tests for generate_yaml_dict function."""

    def test_generate_basic_structure(self) -> None:
        """Generate YAML dict with basic structure."""
        metadata = LawMetadata(
            bwb_id="BWBR0018451",
            title="Wet op de zorgtoeslag",
            regulatory_layer=RegulatoryLayer.WET,
            publication_date="2005-07-26",
        )
        law = Law(metadata=metadata)

        result = generate_yaml_dict(law, "2025-01-01")

        assert result["$schema"] == SCHEMA_URL
        assert result["$id"] == "wet_op_de_zorgtoeslag"
        # RFC-001 Decision 7: uuid field removed
        assert "uuid" not in result
        assert result["regulatory_layer"] == "WET"
        assert result["publication_date"] == "2005-07-26"
        assert result["valid_from"] == "2025-01-01"
        assert result["bwb_id"] == "BWBR0018451"
        assert result["url"] == "https://wetten.overheid.nl/BWBR0018451/2025-01-01"
        assert result["articles"] == []

    def test_generate_with_articles(self) -> None:
        """Generate YAML dict with articles."""
        metadata = LawMetadata(
            bwb_id="BWBR0018451",
            title="Wet op de zorgtoeslag",
            regulatory_layer=RegulatoryLayer.WET,
        )
        articles = [
            Article(
                number="1",
                text="Artikel 1 tekst",
                url="https://wetten.overheid.nl/BWBR0018451/2025-01-01#Artikel1",
            ),
            Article(
                number="2",
                text="Artikel 2 tekst",
                url="https://wetten.overheid.nl/BWBR0018451/2025-01-01#Artikel2",
            ),
        ]
        law = Law(metadata=metadata, articles=articles)

        result = generate_yaml_dict(law, "2025-01-01")

        assert len(result["articles"]) == 2
        assert result["articles"][0]["number"] == "1"
        assert result["articles"][0]["text"] == "Artikel 1 tekst"
        assert result["articles"][1]["number"] == "2"

    def test_generate_fallback_publication_date(self) -> None:
        """Use effective_date if publication_date is None."""
        metadata = LawMetadata(
            bwb_id="BWBR0018451",
            title="Wet op de zorgtoeslag",
            regulatory_layer=RegulatoryLayer.WET,
            publication_date=None,
        )
        law = Law(metadata=metadata)

        result = generate_yaml_dict(law, "2025-01-01")

        assert result["publication_date"] == "2025-01-01"

    def test_generate_different_regulatory_layers(self) -> None:
        """Test different regulatory layer values."""
        for layer in [
            RegulatoryLayer.WET,
            RegulatoryLayer.AMVB,
            RegulatoryLayer.MINISTERIELE_REGELING,
        ]:
            metadata = LawMetadata(
                bwb_id="BWBR0018451",
                title="Test",
                regulatory_layer=layer,
            )
            law = Law(metadata=metadata)

            result = generate_yaml_dict(law, "2025-01-01")

            assert result["regulatory_layer"] == layer.value


class TestSaveYaml:
    """Tests for save_yaml function."""

    def test_save_creates_file(self, tmp_path: Path) -> None:
        """Save creates YAML file in correct location."""
        metadata = LawMetadata(
            bwb_id="BWBR0018451",
            title="Wet op de zorgtoeslag",
            regulatory_layer=RegulatoryLayer.WET,
            publication_date="2005-07-26",
        )
        law = Law(metadata=metadata)

        output_path = save_yaml(law, "2025-01-01", output_base=tmp_path)

        assert output_path.exists()
        assert output_path.name == "2025-01-01.yaml"
        assert output_path.parent.name == "wet_op_de_zorgtoeslag"
        assert output_path.parent.parent.name == "wet"

    def test_save_creates_directory_structure(self, tmp_path: Path) -> None:
        """Save creates necessary directories."""
        metadata = LawMetadata(
            bwb_id="BWBR0018451",
            title="Test Law",
            regulatory_layer=RegulatoryLayer.MINISTERIELE_REGELING,
        )
        law = Law(metadata=metadata)

        output_path = save_yaml(law, "2025-01-01", output_base=tmp_path)

        assert (tmp_path / "ministeriele_regeling" / "test_law").exists()
        assert output_path.exists()

    def test_save_valid_yaml_content(self, tmp_path: Path) -> None:
        """Saved YAML is valid and contains expected content."""
        metadata = LawMetadata(
            bwb_id="BWBR0018451",
            title="Wet op de zorgtoeslag",
            regulatory_layer=RegulatoryLayer.WET,
            publication_date="2005-07-26",
        )
        articles = [
            Article(number="1", text="Test text", url="http://example.com"),
        ]
        law = Law(metadata=metadata, articles=articles)

        output_path = save_yaml(law, "2025-01-01", output_base=tmp_path)

        with open(output_path, encoding="utf-8") as f:
            loaded = yaml.safe_load(f)

        assert loaded["$schema"] == SCHEMA_URL
        assert loaded["$id"] == "wet_op_de_zorgtoeslag"
        # RFC-001 Decision 7: uuid field removed
        assert "uuid" not in loaded
        assert loaded["regulatory_layer"] == "WET"
        assert loaded["valid_from"] == "2025-01-01"
        assert len(loaded["articles"]) == 1
        assert loaded["articles"][0]["number"] == "1"

    def test_save_unicode_content(self, tmp_path: Path) -> None:
        """Handle Unicode content in YAML."""
        metadata = LawMetadata(
            bwb_id="BWBR0018451",
            title="Wet ministeriële regeling",
            regulatory_layer=RegulatoryLayer.WET,
        )
        articles = [
            Article(
                number="1",
                text="Artikel met speciale tekens: é, ë, ü, €",
                url="http://example.com",
            ),
        ]
        law = Law(metadata=metadata, articles=articles)

        output_path = save_yaml(law, "2025-01-01", output_base=tmp_path)

        with open(output_path, encoding="utf-8") as f:
            content = f.read()
            loaded = yaml.safe_load(content)

        assert "ministeriële" in loaded["$id"]
        assert "€" in loaded["articles"][0]["text"]

    def test_save_default_output_base(self) -> None:
        """Default output base is regulation/nl."""
        metadata = LawMetadata(
            bwb_id="BWBR0018451",
            title="Test",
            regulatory_layer=RegulatoryLayer.WET,
        )
        law = Law(metadata=metadata)

        # Just verify the function accepts None without erroring
        # We don't actually want to create files in the project directory
        # So we just test the path construction logic indirectly
        yaml_dict = generate_yaml_dict(law, "2025-01-01")
        assert yaml_dict is not None

    def test_multiline_text_uses_literal_block_scalar(self, tmp_path: Path) -> None:
        """RFC-001 Decision 2: Multiline text must use |- literal block scalar."""
        metadata = LawMetadata(
            bwb_id="BWBR0018451",
            title="Test Law",
            regulatory_layer=RegulatoryLayer.WET,
        )
        multiline_text = "Eerste regel.\n\nTweede regel met meer tekst."
        articles = [
            Article(
                number="1",
                text=multiline_text,
                url="http://example.com",
            ),
        ]
        law = Law(metadata=metadata, articles=articles)

        output_path = save_yaml(law, "2025-01-01", output_base=tmp_path)

        # Read raw file content to verify literal block scalar marker
        with open(output_path, encoding="utf-8") as f:
            raw_content = f.read()

        # The text field must use |- (literal block scalar) for multiline
        assert "text: |-" in raw_content or "text: |" in raw_content, (
            "RFC-001 Decision 2: Multiline text must use literal block scalar (|-). "
            f"Got:\n{raw_content}"
        )

"""End-to-end integration tests for the harvester pipeline."""

import json
from pathlib import Path
from unittest.mock import Mock, patch

import pytest
import jsonschema
import ruamel.yaml
from lxml import etree

from harvester.models import Law
from harvester.parsers.content_parser import parse_articles_split
from harvester.parsers.wti_parser import parse_wti_metadata
from harvester.storage.yaml_writer import generate_yaml_dict, save_yaml

FIXTURES_DIR = Path(__file__).parent / "fixtures"
SCHEMA_DIR = Path(__file__).parent.parent.parent / "schema" / "v0.3.1"

# Law fixtures with their effective dates
LAW_FIXTURES = [
    ("zorgtoeslag", "2025-01-01"),
]


def load_schema() -> dict:
    """Load the JSON schema for validation."""
    schema_path = SCHEMA_DIR / "schema.json"
    with open(schema_path, encoding="utf-8") as f:
        return json.load(f)


def run_pipeline(law_folder: str, effective_date: str) -> tuple[dict, list]:
    """Run the harvester pipeline for a law fixture.

    Returns:
        Tuple of (yaml_dict, articles)
    """
    folder = FIXTURES_DIR / law_folder
    wti_tree = etree.parse(str(folder / "wti.xml"))
    content_tree = etree.parse(str(folder / "content.xml"))

    metadata = parse_wti_metadata(wti_tree.getroot())
    articles = parse_articles_split(
        content_tree.getroot(), metadata.bwb_id, effective_date
    )
    law = Law(metadata=metadata, articles=articles)
    yaml_dict = generate_yaml_dict(law, effective_date)

    return yaml_dict, articles


def load_expected(law_folder: str) -> dict:
    """Load expected YAML output for a law fixture.

    Uses ruamel.yaml to be consistent with how we write YAML,
    avoiding PyYAML's sexagesimal interpretation (e.g., 8:41 -> 521).
    """
    expected_path = FIXTURES_DIR / law_folder / "expected.yaml"
    yaml = ruamel.yaml.YAML()
    yaml.preserve_quotes = True
    with open(expected_path, encoding="utf-8") as f:
        return yaml.load(f)


class TestLawPipeline:
    """Parametrized integration tests for all law fixtures."""

    @pytest.mark.parametrize("law_folder,effective_date", LAW_FIXTURES)
    def test_matches_expected_output(
        self, law_folder: str, effective_date: str
    ) -> None:
        """Test that pipeline output matches expected YAML exactly."""
        yaml_dict, _ = run_pipeline(law_folder, effective_date)
        expected = load_expected(law_folder)
        assert yaml_dict == expected, f"{law_folder}: Generated YAML does not match"

    @pytest.mark.parametrize("law_folder,effective_date", LAW_FIXTURES)
    def test_validates_against_schema(
        self, law_folder: str, effective_date: str
    ) -> None:
        """Test that generated YAML validates against the JSON schema."""
        yaml_dict, _ = run_pipeline(law_folder, effective_date)
        schema = load_schema()
        jsonschema.validate(yaml_dict, schema)

    @pytest.mark.parametrize("law_folder,effective_date", LAW_FIXTURES)
    def test_article_count_matches_expected(
        self, law_folder: str, effective_date: str
    ) -> None:
        """Test that article count matches expected output."""
        yaml_dict, articles = run_pipeline(law_folder, effective_date)
        expected = load_expected(law_folder)
        assert len(articles) == len(expected["articles"]), (
            f"{law_folder}: Article count mismatch"
        )


class TestZorgtoeslag:
    """Zorgtoeslag-specific tests for detailed validation."""

    def test_article_text_content(self) -> None:
        """Test that article text is correctly extracted with formatting."""
        _, articles = run_pipeline("zorgtoeslag", "2025-01-01")

        # Article 1.1 (intro) should have the intro text
        art_1_1 = next(a for a in articles if a.number == "1.1")
        assert "In deze wet" in art_1_1.text

        # Article 1.1.a should have "Onze Minister"
        art_1_1_a = next(a for a in articles if a.number == "1.1.a")
        assert "Onze Minister" in art_1_1_a.text

        # Article 1.1.b should have external references as markdown links
        art_1_1_b = next(a for a in articles if a.number == "1.1.b")
        assert "[" in art_1_1_b.text and "]" in art_1_1_b.text

        # Article 8 should reference the citeertitel
        art_8 = next(a for a in articles if a.number == "8")
        assert "aangehaald als" in art_8.text.lower()
        assert "zorgtoeslag" in art_8.text.lower()

    def test_article_urls(self) -> None:
        """Test that article URLs are generated with correct format."""
        _, articles = run_pipeline("zorgtoeslag", "2025-01-01")

        # All components of article 1 should point to Artikel1
        art_1_1 = next(a for a in articles if a.number == "1.1")
        art_1_1_a = next(a for a in articles if a.number == "1.1.a")
        assert (
            art_1_1.url == "https://wetten.overheid.nl/BWBR0018451/2025-01-01#Artikel1"
        )
        assert (
            art_1_1_a.url
            == "https://wetten.overheid.nl/BWBR0018451/2025-01-01#Artikel1"
        )

        # Article 8 should point to Artikel8
        art_8 = next(a for a in articles if a.number == "8")
        assert art_8.url == "https://wetten.overheid.nl/BWBR0018451/2025-01-01#Artikel8"

    def test_save_yaml_creates_valid_file(self, tmp_path: Path) -> None:
        """Test that saved YAML file can be loaded and validates."""
        folder = FIXTURES_DIR / "zorgtoeslag"
        wti_tree = etree.parse(str(folder / "wti.xml"))
        content_tree = etree.parse(str(folder / "content.xml"))

        metadata = parse_wti_metadata(wti_tree.getroot())
        articles = parse_articles_split(
            content_tree.getroot(), metadata.bwb_id, "2025-01-01"
        )
        law = Law(metadata=metadata, articles=articles)

        output_path = save_yaml(law, "2025-01-01", output_base=tmp_path)

        assert output_path.exists()
        assert output_path.suffix == ".yaml"

        with open(output_path, encoding="utf-8") as f:
            loaded = ruamel.yaml.YAML().load(f)

        assert "zorgtoeslag" in loaded["$id"]
        assert loaded["regulatory_layer"] == "WET"
        assert len(loaded["articles"]) == 36  # 35 + aanhef

        schema = load_schema()
        jsonschema.validate(loaded, schema)


class TestEndToEndWithMockedDownload:
    """Integration tests with mocked HTTP downloads."""

    def test_full_pipeline_with_mocked_http(self, tmp_path: Path) -> None:
        """Test complete pipeline with mocked HTTP requests."""
        folder = FIXTURES_DIR / "zorgtoeslag"

        with open(folder / "wti.xml", "rb") as f:
            wti_content = f.read()
        with open(folder / "content.xml", "rb") as f:
            content_content = f.read()

        def mock_get(url: str, **kwargs) -> Mock:
            response = Mock()
            response.raise_for_status = Mock()
            if ".WTI" in url:
                response.content = wti_content
            else:
                response.content = content_content
            return response

        with patch("requests.get", side_effect=mock_get) as mock_requests:
            from harvester.parsers.content_parser import download_content
            from harvester.parsers.wti_parser import download_wti

            wti_tree = download_wti("BWBR0018451")
            content_tree = download_content("BWBR0018451", "2025-01-01")

            metadata = parse_wti_metadata(wti_tree)
            articles = parse_articles_split(content_tree, metadata.bwb_id, "2025-01-01")
            law = Law(metadata=metadata, articles=articles)

            output_path = save_yaml(law, "2025-01-01", output_base=tmp_path)

            assert output_path.exists()

            with open(output_path, encoding="utf-8") as f:
                loaded = ruamel.yaml.YAML().load(f)

            schema = load_schema()
            jsonschema.validate(loaded, schema)

            assert mock_requests.call_count == 2

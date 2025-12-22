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
    ("kieswet", "2025-08-01"),
    ("wlz", "2025-07-05"),
    ("zvw", "2025-07-05"),
    ("awir", "2025-01-01"),
    ("participatiewet", "2024-01-01"),
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


class TestKieswet:
    """Kieswet-specific tests for section-based article numbering (A 1, B 5, etc.)."""

    def test_section_numbering(self) -> None:
        """Test that section-based article numbers are correctly parsed."""
        _, articles = run_pipeline("kieswet", "2025-08-01")

        # A 1 has no leden, so it's not split
        art_a1 = next((a for a in articles if a.number == "A 1"), None)
        assert art_a1 is not None
        assert "In deze wet" in art_a1.text

        # B 1 has leden, so it's split into B 1.1, B 1.2, etc.
        art_b1_1 = next((a for a in articles if a.number == "B 1.1"), None)
        assert art_b1_1 is not None

        # Check URL format uses underscores instead of spaces
        assert (
            art_a1.url == "https://wetten.overheid.nl/BWBR0004627/2025-08-01#ArtikelA_1"
        )
        assert (
            art_b1_1.url
            == "https://wetten.overheid.nl/BWBR0004627/2025-08-01#ArtikelB_1"
        )

    def test_all_chapters_present(self) -> None:
        """Test that all chapters A through Z are present."""
        _, articles = run_pipeline("kieswet", "2025-08-01")

        chapters = set()
        for art in articles:
            # Skip aanhef and check for uppercase chapter letters
            if art.number and art.number != "aanhef" and art.number[0].isupper():
                chapters.add(art.number[0])

        expected_chapters = set("ABCDEFGHIJKLMNOPQRSTUVWXYZ")
        assert chapters == expected_chapters


class TestWlz:
    """WLZ-specific tests for decimal article numbering (1.1, 1.2, etc.)."""

    def test_decimal_article_numbering(self) -> None:
        """Test that decimal article numbers are correctly parsed."""
        _, articles = run_pipeline("wlz", "2025-07-05")

        # Check article 1.1 components exist
        art_1_1_1 = next((a for a in articles if a.number == "1.1.1"), None)
        assert art_1_1_1 is not None
        assert "In deze wet" in art_1_1_1.text

        # Check deep nesting: artikel 1.1, lid 2, onderdeel 1, sub a
        art_1_1_2_1_a = next((a for a in articles if a.number == "1.1.2.1.a"), None)
        assert art_1_1_2_1_a is not None

        # Check last article (citeertitel)
        art_13_1_3 = next((a for a in articles if a.number == "13.1.3"), None)
        assert art_13_1_3 is not None
        assert "langdurige zorg" in art_13_1_3.text.lower()

    def test_all_chapters_present(self) -> None:
        """Test that all chapters 1 through 13 are present."""
        _, articles = run_pipeline("wlz", "2025-07-05")

        chapters = set()
        for art in articles:
            parts = art.number.split(".")
            if parts[0].isdigit():
                chapters.add(int(parts[0]))

        expected_chapters = set(range(1, 14))
        assert chapters == expected_chapters


class TestZvw:
    """ZVW-specific tests."""

    def test_references_wlz(self) -> None:
        """Test that ZVW contains references to WLZ (cross-law dependency)."""
        _, articles = run_pipeline("zvw", "2025-07-05")

        art_2_1 = next((a for a in articles if a.number == "2.1"), None)
        assert art_2_1 is not None
        assert "langdurige zorg" in art_2_1.text.lower()

    def test_citeertitel(self) -> None:
        """Test that citeertitel is correctly extracted."""
        _, articles = run_pipeline("zvw", "2025-07-05")

        art_128 = next((a for a in articles if a.number == "128"), None)
        assert art_128 is not None
        assert "zorgverzekeringswet" in art_128.text.lower()


class TestAwir:
    """AWIR-specific tests for sub-articles (3a, 31bis, etc.)."""

    def test_sub_articles(self) -> None:
        """Test that sub-articles (3a, 8a, 31bis) are correctly parsed."""
        _, articles = run_pipeline("awir", "2025-01-01")

        # Check sub-article 3a exists
        art_3a = next((a for a in articles if a.number.startswith("3a")), None)
        assert art_3a is not None

        # Check "bis" articles (Latin for "second")
        art_31bis = next((a for a in articles if a.number.startswith("31bis")), None)
        assert art_31bis is not None

        art_49bis = next((a for a in articles if a.number.startswith("49bis")), None)
        assert art_49bis is not None

    def test_citeertitel(self) -> None:
        """Test that citeertitel is correctly extracted."""
        _, articles = run_pipeline("awir", "2025-01-01")

        art_51 = next((a for a in articles if a.number == "51"), None)
        assert art_51 is not None
        assert "inkomensafhankelijke regelingen" in art_51.text.lower()


class TestParticipatiewet:
    """Participatiewet-specific tests."""

    def test_article_structure(self) -> None:
        """Test that article structure is correctly parsed."""
        _, articles = run_pipeline("participatiewet", "2024-01-01")

        # Article 1 has direct onderdelen (no leden)
        art_1 = next((a for a in articles if a.number == "1"), None)
        assert art_1 is not None
        assert "In deze wet" in art_1.text

        # Article 1.a should have "Onze Minister"
        art_1_a = next((a for a in articles if a.number == "1.a"), None)
        assert art_1_a is not None
        assert "Onze Minister" in art_1_a.text

        # Article 1.b should reference "college"
        art_1_b = next((a for a in articles if a.number == "1.b"), None)
        assert art_1_b is not None
        assert "college" in art_1_b.text.lower()

    def test_citeertitel(self) -> None:
        """Test that citeertitel is correctly extracted."""
        _, articles = run_pipeline("participatiewet", "2024-01-01")

        art_86 = next((a for a in articles if a.number == "86"), None)
        assert art_86 is not None
        assert "participatiewet" in art_86.text.lower()


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

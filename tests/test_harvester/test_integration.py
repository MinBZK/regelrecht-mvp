"""End-to-end integration tests for the harvester pipeline."""

import json
from pathlib import Path
from unittest.mock import Mock, patch

import jsonschema
import yaml
from lxml import etree

from harvester.models import Law, RegulatoryLayer
from harvester.parsers.content_parser import parse_articles_split
from harvester.parsers.wti_parser import parse_wti_metadata
from harvester.storage.yaml_writer import generate_yaml_dict, save_yaml

FIXTURES_DIR = Path(__file__).parent / "fixtures"
SCHEMA_DIR = Path(__file__).parent.parent.parent / "schema" / "v0.2.0"


def load_schema() -> dict:
    """Load the JSON schema for validation."""
    schema_path = SCHEMA_DIR / "schema.json"
    with open(schema_path, encoding="utf-8") as f:
        return json.load(f)


def load_expected_yaml(filename: str) -> dict:
    """Load expected YAML output for comparison."""
    expected_path = FIXTURES_DIR / filename
    with open(expected_path, encoding="utf-8") as f:
        return yaml.safe_load(f)


class TestEndToEndPipeline:
    """Integration tests for the complete harvester pipeline."""

    def test_full_pipeline_matches_expected_output(self) -> None:
        """Test that pipeline output matches expected YAML exactly."""
        # Load fixtures (real XML from BWB repository)
        wti_path = FIXTURES_DIR / "zorgtoeslag_wti.xml"
        toestand_path = FIXTURES_DIR / "zorgtoeslag_toestand.xml"

        wti_tree = etree.parse(str(wti_path))
        toestand_tree = etree.parse(str(toestand_path))

        # Run pipeline with article splitting
        metadata = parse_wti_metadata(wti_tree.getroot())
        articles = parse_articles_split(
            toestand_tree.getroot(), metadata.bwb_id, "2025-01-01"
        )
        law = Law(metadata=metadata, articles=articles)
        yaml_dict = generate_yaml_dict(law, "2025-01-01")

        # Load expected output
        expected = load_expected_yaml("zorgtoeslag_expected.yaml")

        # Compare entire output
        assert yaml_dict == expected, "Generated YAML does not match expected output"

    def test_full_pipeline_wti_to_yaml(self) -> None:
        """Test complete pipeline: WTI XML + Toestand XML -> YAML output."""
        # Load fixtures (real XML from BWB repository)
        wti_path = FIXTURES_DIR / "zorgtoeslag_wti.xml"
        toestand_path = FIXTURES_DIR / "zorgtoeslag_toestand.xml"

        wti_tree = etree.parse(str(wti_path))
        toestand_tree = etree.parse(str(toestand_path))

        # Parse metadata from WTI
        metadata = parse_wti_metadata(wti_tree.getroot())

        assert metadata.bwb_id == "BWBR0018451"
        assert "zorgtoeslag" in metadata.title.lower()
        assert metadata.regulatory_layer == RegulatoryLayer.WET
        assert metadata.publication_date is not None

        # Parse articles from Toestand (split to components)
        effective_date = "2025-01-01"
        articles = parse_articles_split(
            toestand_tree.getroot(), metadata.bwb_id, effective_date
        )

        # Zorgtoeslag wet (2025 version) has 35 article components when split:
        # Art 1: 10, Art 2: 7, Art 3: 5, Art 4: 1, Art 4a: 4, Art 5: 5, Art 6-8: 3
        assert len(articles) == 35
        assert articles[0].number == "1.1"  # First component is lid 1 intro
        assert articles[-1].number == "8"  # Last is artikel 8

        # Create Law object
        law = Law(metadata=metadata, articles=articles, uuid="test-uuid-12345678")

        # Generate YAML dict
        yaml_dict = generate_yaml_dict(law, effective_date)

        # Verify structure
        assert "zorgtoeslag" in yaml_dict["$id"]
        assert yaml_dict["regulatory_layer"] == "WET"
        assert yaml_dict["bwb_id"] == "BWBR0018451"
        assert yaml_dict["url"] == "https://wetten.overheid.nl/BWBR0018451/2025-01-01"
        assert len(yaml_dict["articles"]) == 35

    def test_yaml_validates_against_schema(self) -> None:
        """Test that generated YAML validates against the JSON schema."""
        # Load fixtures
        wti_path = FIXTURES_DIR / "zorgtoeslag_wti.xml"
        toestand_path = FIXTURES_DIR / "zorgtoeslag_toestand.xml"

        wti_tree = etree.parse(str(wti_path))
        toestand_tree = etree.parse(str(toestand_path))

        # Run pipeline with article splitting
        metadata = parse_wti_metadata(wti_tree.getroot())
        articles = parse_articles_split(
            toestand_tree.getroot(), metadata.bwb_id, "2025-01-01"
        )
        law = Law(metadata=metadata, articles=articles)
        yaml_dict = generate_yaml_dict(law, "2025-01-01")

        # Load schema and validate
        schema = load_schema()
        jsonschema.validate(yaml_dict, schema)  # Raises if invalid

    def test_article_text_contains_expected_content(self) -> None:
        """Test that article text is correctly extracted with formatting."""
        toestand_path = FIXTURES_DIR / "zorgtoeslag_toestand.xml"
        toestand_tree = etree.parse(str(toestand_path))

        articles = parse_articles_split(
            toestand_tree.getroot(), "BWBR0018451", "2025-01-01"
        )

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

    def test_article_urls_generated_correctly(self) -> None:
        """Test that article URLs are generated with correct format."""
        toestand_path = FIXTURES_DIR / "zorgtoeslag_toestand.xml"
        toestand_tree = etree.parse(str(toestand_path))

        articles = parse_articles_split(
            toestand_tree.getroot(), "BWBR0018451", "2025-01-01"
        )

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
        # Load fixtures
        wti_path = FIXTURES_DIR / "zorgtoeslag_wti.xml"
        toestand_path = FIXTURES_DIR / "zorgtoeslag_toestand.xml"

        wti_tree = etree.parse(str(wti_path))
        toestand_tree = etree.parse(str(toestand_path))

        # Run pipeline with article splitting
        metadata = parse_wti_metadata(wti_tree.getroot())
        articles = parse_articles_split(
            toestand_tree.getroot(), metadata.bwb_id, "2025-01-01"
        )
        law = Law(metadata=metadata, articles=articles)

        # Save to file
        output_path = save_yaml(law, "2025-01-01", output_base=tmp_path)

        # Verify file exists and can be loaded
        assert output_path.exists()
        assert output_path.suffix == ".yaml"

        with open(output_path, encoding="utf-8") as f:
            loaded = yaml.safe_load(f)

        # Validate loaded content
        assert "zorgtoeslag" in loaded["$id"]
        assert loaded["regulatory_layer"] == "WET"
        assert len(loaded["articles"]) == 35  # Split articles

        # Validate against schema
        schema = load_schema()
        jsonschema.validate(loaded, schema)


class TestEndToEndWithMockedDownload:
    """Integration tests with mocked HTTP downloads."""

    def test_full_pipeline_with_mocked_http(self, tmp_path: Path) -> None:
        """Test complete pipeline with mocked HTTP requests."""
        # Load fixture content
        wti_path = FIXTURES_DIR / "zorgtoeslag_wti.xml"
        toestand_path = FIXTURES_DIR / "zorgtoeslag_toestand.xml"

        with open(wti_path, "rb") as f:
            wti_content = f.read()
        with open(toestand_path, "rb") as f:
            toestand_content = f.read()

        # Create responses for different URLs
        def mock_get(url: str, **kwargs) -> Mock:
            response = Mock()
            response.raise_for_status = Mock()
            if ".WTI" in url:
                response.content = wti_content
            else:
                response.content = toestand_content
            return response

        # Mock HTTP at requests module level
        with patch("requests.get", side_effect=mock_get) as mock_requests:
            from harvester.parsers.content_parser import download_content
            from harvester.parsers.wti_parser import download_wti

            wti_tree = download_wti("BWBR0018451")
            content_tree = download_content("BWBR0018451", "2025-01-01")

            # Parse and generate with article splitting
            metadata = parse_wti_metadata(wti_tree)
            articles = parse_articles_split(content_tree, metadata.bwb_id, "2025-01-01")
            law = Law(metadata=metadata, articles=articles)

            # Save and validate
            output_path = save_yaml(law, "2025-01-01", output_base=tmp_path)

            assert output_path.exists()

            with open(output_path, encoding="utf-8") as f:
                loaded = yaml.safe_load(f)

            schema = load_schema()
            jsonschema.validate(loaded, schema)

            # Verify requests were made
            assert mock_requests.call_count == 2


class TestKieswetPipeline:
    """Integration tests for Kieswet with section-based article numbering (A 1, B 5, etc.)."""

    def test_kieswet_matches_expected_output(self) -> None:
        """Test that Kieswet pipeline output matches expected YAML exactly."""
        wti_path = FIXTURES_DIR / "kieswet_wti.xml"
        toestand_path = FIXTURES_DIR / "kieswet_toestand.xml"

        wti_tree = etree.parse(str(wti_path))
        toestand_tree = etree.parse(str(toestand_path))

        metadata = parse_wti_metadata(wti_tree.getroot())
        articles = parse_articles_split(
            toestand_tree.getroot(), metadata.bwb_id, "2025-08-01"
        )
        law = Law(metadata=metadata, articles=articles)
        yaml_dict = generate_yaml_dict(law, "2025-08-01")

        expected = load_expected_yaml("kieswet_expected.yaml")
        assert yaml_dict == expected, "Generated YAML does not match expected output"

    def test_kieswet_validates_against_schema(self) -> None:
        """Test that generated Kieswet YAML validates against the JSON schema."""
        wti_path = FIXTURES_DIR / "kieswet_wti.xml"
        toestand_path = FIXTURES_DIR / "kieswet_toestand.xml"

        wti_tree = etree.parse(str(wti_path))
        toestand_tree = etree.parse(str(toestand_path))

        metadata = parse_wti_metadata(wti_tree.getroot())
        articles = parse_articles_split(
            toestand_tree.getroot(), metadata.bwb_id, "2025-08-01"
        )
        law = Law(metadata=metadata, articles=articles)
        yaml_dict = generate_yaml_dict(law, "2025-08-01")

        schema = load_schema()
        jsonschema.validate(yaml_dict, schema)

    def test_kieswet_article_count(self) -> None:
        """Test that Kieswet produces expected number of article components."""
        toestand_path = FIXTURES_DIR / "kieswet_toestand.xml"
        toestand_tree = etree.parse(str(toestand_path))

        articles = parse_articles_split(
            toestand_tree.getroot(), "BWBR0004627", "2025-08-01"
        )

        # Kieswet has 631 articles that split into 1810 components (including unmarked definition lists)
        assert len(articles) == 1810

    def test_kieswet_section_numbering(self) -> None:
        """Test that section-based article numbers (A 1, B 5) are correctly parsed."""
        toestand_path = FIXTURES_DIR / "kieswet_toestand.xml"
        toestand_tree = etree.parse(str(toestand_path))

        articles = parse_articles_split(
            toestand_tree.getroot(), "BWBR0004627", "2025-08-01"
        )

        # Check first articles from different chapters
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

    def test_kieswet_all_chapters_present(self) -> None:
        """Test that all chapters A through Z are present."""
        toestand_path = FIXTURES_DIR / "kieswet_toestand.xml"
        toestand_tree = etree.parse(str(toestand_path))

        articles = parse_articles_split(
            toestand_tree.getroot(), "BWBR0004627", "2025-08-01"
        )

        # Get unique chapter letters
        chapters = set()
        for art in articles:
            if art.number and art.number[0].isalpha():
                chapters.add(art.number[0])

        # Kieswet has chapters A through Z
        expected_chapters = set("ABCDEFGHIJKLMNOPQRSTUVWXYZ")
        assert chapters == expected_chapters


class TestWlzPipeline:
    """Integration tests for Wet langdurige zorg with decimal article numbering (1.1, 1.2, etc.)."""

    def test_wlz_matches_expected_output(self) -> None:
        """Test that WLZ pipeline output matches expected YAML exactly."""
        wti_path = FIXTURES_DIR / "wlz_wti.xml"
        toestand_path = FIXTURES_DIR / "wlz_toestand.xml"

        wti_tree = etree.parse(str(wti_path))
        toestand_tree = etree.parse(str(toestand_path))

        metadata = parse_wti_metadata(wti_tree.getroot())
        articles = parse_articles_split(
            toestand_tree.getroot(), metadata.bwb_id, "2025-07-05"
        )
        law = Law(metadata=metadata, articles=articles)
        yaml_dict = generate_yaml_dict(law, "2025-07-05")

        expected = load_expected_yaml("wlz_expected.yaml")
        assert yaml_dict == expected, "Generated YAML does not match expected output"

    def test_wlz_validates_against_schema(self) -> None:
        """Test that generated WLZ YAML validates against the JSON schema."""
        wti_path = FIXTURES_DIR / "wlz_wti.xml"
        toestand_path = FIXTURES_DIR / "wlz_toestand.xml"

        wti_tree = etree.parse(str(wti_path))
        toestand_tree = etree.parse(str(toestand_path))

        metadata = parse_wti_metadata(wti_tree.getroot())
        articles = parse_articles_split(
            toestand_tree.getroot(), metadata.bwb_id, "2025-07-05"
        )
        law = Law(metadata=metadata, articles=articles)
        yaml_dict = generate_yaml_dict(law, "2025-07-05")

        schema = load_schema()
        jsonschema.validate(yaml_dict, schema)

    def test_wlz_article_count(self) -> None:
        """Test that WLZ produces expected number of article components."""
        toestand_path = FIXTURES_DIR / "wlz_toestand.xml"
        toestand_tree = etree.parse(str(toestand_path))

        articles = parse_articles_split(
            toestand_tree.getroot(), "BWBR0035917", "2025-07-05"
        )

        # WLZ has 732 article components (including unmarked definition lists)
        assert len(articles) == 732

    def test_wlz_decimal_article_numbering(self) -> None:
        """Test that decimal article numbers (1.1, 1.2) are correctly parsed."""
        toestand_path = FIXTURES_DIR / "wlz_toestand.xml"
        toestand_tree = etree.parse(str(toestand_path))

        articles = parse_articles_split(
            toestand_tree.getroot(), "BWBR0035917", "2025-07-05"
        )

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

    def test_wlz_all_chapters_present(self) -> None:
        """Test that all chapters 1 through 13 are present."""
        toestand_path = FIXTURES_DIR / "wlz_toestand.xml"
        toestand_tree = etree.parse(str(toestand_path))

        articles = parse_articles_split(
            toestand_tree.getroot(), "BWBR0035917", "2025-07-05"
        )

        # Get unique chapter numbers (first part before dot)
        chapters = set()
        for art in articles:
            parts = art.number.split(".")
            if parts[0].isdigit():
                chapters.add(int(parts[0]))

        # WLZ has chapters 1 through 13
        expected_chapters = set(range(1, 14))
        assert chapters == expected_chapters


class TestZvwPipeline:
    """Integration tests for Zorgverzekeringswet with standard article numbering."""

    def test_zvw_matches_expected_output(self) -> None:
        """Test that ZVW pipeline output matches expected YAML exactly."""
        wti_path = FIXTURES_DIR / "zvw_wti.xml"
        toestand_path = FIXTURES_DIR / "zvw_toestand.xml"

        wti_tree = etree.parse(str(wti_path))
        toestand_tree = etree.parse(str(toestand_path))

        metadata = parse_wti_metadata(wti_tree.getroot())
        articles = parse_articles_split(
            toestand_tree.getroot(), metadata.bwb_id, "2025-07-05"
        )
        law = Law(metadata=metadata, articles=articles)
        yaml_dict = generate_yaml_dict(law, "2025-07-05")

        expected = load_expected_yaml("zvw_expected.yaml")
        assert yaml_dict == expected, "Generated YAML does not match expected output"

    def test_zvw_validates_against_schema(self) -> None:
        """Test that generated ZVW YAML validates against the JSON schema."""
        wti_path = FIXTURES_DIR / "zvw_wti.xml"
        toestand_path = FIXTURES_DIR / "zvw_toestand.xml"

        wti_tree = etree.parse(str(wti_path))
        toestand_tree = etree.parse(str(toestand_path))

        metadata = parse_wti_metadata(wti_tree.getroot())
        articles = parse_articles_split(
            toestand_tree.getroot(), metadata.bwb_id, "2025-07-05"
        )
        law = Law(metadata=metadata, articles=articles)
        yaml_dict = generate_yaml_dict(law, "2025-07-05")

        schema = load_schema()
        jsonschema.validate(yaml_dict, schema)

    def test_zvw_article_count(self) -> None:
        """Test that ZVW produces expected number of article components."""
        toestand_path = FIXTURES_DIR / "zvw_toestand.xml"
        toestand_tree = etree.parse(str(toestand_path))

        articles = parse_articles_split(
            toestand_tree.getroot(), "BWBR0018450", "2025-07-05"
        )

        # ZVW has 800 article components (including unmarked definition lists)
        assert len(articles) == 800

    def test_zvw_references_wlz(self) -> None:
        """Test that ZVW contains references to WLZ (cross-law dependency)."""
        toestand_path = FIXTURES_DIR / "zvw_toestand.xml"
        toestand_tree = etree.parse(str(toestand_path))

        articles = parse_articles_split(
            toestand_tree.getroot(), "BWBR0018450", "2025-07-05"
        )

        # Article 2.1 should reference Wet langdurige zorg
        art_2_1 = next((a for a in articles if a.number == "2.1"), None)
        assert art_2_1 is not None
        assert "langdurige zorg" in art_2_1.text.lower()

    def test_zvw_citeertitel(self) -> None:
        """Test that citeertitel is correctly extracted."""
        toestand_path = FIXTURES_DIR / "zvw_toestand.xml"
        toestand_tree = etree.parse(str(toestand_path))

        articles = parse_articles_split(
            toestand_tree.getroot(), "BWBR0018450", "2025-07-05"
        )

        # Article 128 should contain citeertitel
        art_128 = next((a for a in articles if a.number == "128"), None)
        assert art_128 is not None
        assert "zorgverzekeringswet" in art_128.text.lower()


class TestAwirPipeline:
    """Integration tests for AWIR with sub-articles (3a, 31bis, etc.)."""

    def test_awir_matches_expected_output(self) -> None:
        """Test that AWIR pipeline output matches expected YAML exactly."""
        wti_path = FIXTURES_DIR / "awir_wti.xml"
        toestand_path = FIXTURES_DIR / "awir_toestand.xml"

        wti_tree = etree.parse(str(wti_path))
        toestand_tree = etree.parse(str(toestand_path))

        metadata = parse_wti_metadata(wti_tree.getroot())
        articles = parse_articles_split(
            toestand_tree.getroot(), metadata.bwb_id, "2025-01-01"
        )
        law = Law(metadata=metadata, articles=articles)
        yaml_dict = generate_yaml_dict(law, "2025-01-01")

        expected = load_expected_yaml("awir_expected.yaml")
        assert yaml_dict == expected, "Generated YAML does not match expected output"

    def test_awir_validates_against_schema(self) -> None:
        """Test that generated AWIR YAML validates against the JSON schema."""
        wti_path = FIXTURES_DIR / "awir_wti.xml"
        toestand_path = FIXTURES_DIR / "awir_toestand.xml"

        wti_tree = etree.parse(str(wti_path))
        toestand_tree = etree.parse(str(toestand_path))

        metadata = parse_wti_metadata(wti_tree.getroot())
        articles = parse_articles_split(
            toestand_tree.getroot(), metadata.bwb_id, "2025-01-01"
        )
        law = Law(metadata=metadata, articles=articles)
        yaml_dict = generate_yaml_dict(law, "2025-01-01")

        schema = load_schema()
        jsonschema.validate(yaml_dict, schema)

    def test_awir_article_count(self) -> None:
        """Test that AWIR produces expected number of article components."""
        toestand_path = FIXTURES_DIR / "awir_toestand.xml"
        toestand_tree = etree.parse(str(toestand_path))

        articles = parse_articles_split(
            toestand_tree.getroot(), "BWBR0018472", "2025-01-01"
        )

        # AWIR has 346 article components (including unmarked definition lists)
        assert len(articles) == 346

    def test_awir_sub_articles(self) -> None:
        """Test that sub-articles (3a, 8a, 31bis) are correctly parsed."""
        toestand_path = FIXTURES_DIR / "awir_toestand.xml"
        toestand_tree = etree.parse(str(toestand_path))

        articles = parse_articles_split(
            toestand_tree.getroot(), "BWBR0018472", "2025-01-01"
        )

        # Check sub-article 3a exists
        art_3a = next((a for a in articles if a.number.startswith("3a")), None)
        assert art_3a is not None

        # Check "bis" articles (Latin for "second")
        art_31bis = next((a for a in articles if a.number.startswith("31bis")), None)
        assert art_31bis is not None

        art_49bis = next((a for a in articles if a.number.startswith("49bis")), None)
        assert art_49bis is not None

    def test_awir_citeertitel(self) -> None:
        """Test that citeertitel is correctly extracted."""
        toestand_path = FIXTURES_DIR / "awir_toestand.xml"
        toestand_tree = etree.parse(str(toestand_path))

        articles = parse_articles_split(
            toestand_tree.getroot(), "BWBR0018472", "2025-01-01"
        )

        # Article 51 should contain citeertitel
        art_51 = next((a for a in articles if a.number == "51"), None)
        assert art_51 is not None
        assert "inkomensafhankelijke regelingen" in art_51.text.lower()

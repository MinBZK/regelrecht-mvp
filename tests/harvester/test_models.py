"""Tests for harvester data models."""

from harvester.models import Article, Law, LawMetadata, RegulatoryLayer


class TestRegulatoryLayer:
    """Tests for the RegulatoryLayer enum."""

    def test_wet_value(self) -> None:
        assert RegulatoryLayer.WET.value == "WET"

    def test_amvb_value(self) -> None:
        assert RegulatoryLayer.AMVB.value == "AMVB"

    def test_ministeriele_regeling_value(self) -> None:
        assert RegulatoryLayer.MINISTERIELE_REGELING.value == "MINISTERIELE_REGELING"

    def test_enum_is_string(self) -> None:
        # RegulatoryLayer inherits from str
        assert isinstance(RegulatoryLayer.WET, str)
        assert RegulatoryLayer.WET == "WET"


class TestLawMetadata:
    """Tests for the LawMetadata dataclass."""

    def test_basic_creation(self) -> None:
        metadata = LawMetadata(
            bwb_id="BWBR0018451",
            title="Wet op de zorgtoeslag",
            regulatory_layer=RegulatoryLayer.WET,
        )
        assert metadata.bwb_id == "BWBR0018451"
        assert metadata.title == "Wet op de zorgtoeslag"
        assert metadata.regulatory_layer == RegulatoryLayer.WET
        assert metadata.publication_date is None
        assert metadata.effective_date is None

    def test_creation_with_dates(self) -> None:
        metadata = LawMetadata(
            bwb_id="BWBR0018451",
            title="Wet op de zorgtoeslag",
            regulatory_layer=RegulatoryLayer.WET,
            publication_date="2005-07-26",
            effective_date="2006-01-01",
        )
        assert metadata.publication_date == "2005-07-26"
        assert metadata.effective_date == "2006-01-01"

    def test_to_slug_simple(self) -> None:
        metadata = LawMetadata(
            bwb_id="BWBR0018451",
            title="Wet op de zorgtoeslag",
            regulatory_layer=RegulatoryLayer.WET,
        )
        assert metadata.to_slug() == "wet_op_de_zorgtoeslag"

    def test_to_slug_with_special_characters(self) -> None:
        metadata = LawMetadata(
            bwb_id="BWBR0012345",
            title="Wet (bijzondere) bepalingen!",
            regulatory_layer=RegulatoryLayer.WET,
        )
        assert metadata.to_slug() == "wet_bijzondere_bepalingen"

    def test_to_slug_with_dashes(self) -> None:
        metadata = LawMetadata(
            bwb_id="BWBR0012345",
            title="Wet basis-registratie personen",
            regulatory_layer=RegulatoryLayer.WET,
        )
        assert metadata.to_slug() == "wet_basis_registratie_personen"

    def test_to_slug_uppercase(self) -> None:
        metadata = LawMetadata(
            bwb_id="BWBR0012345",
            title="WET OP DE ZORGTOESLAG",
            regulatory_layer=RegulatoryLayer.WET,
        )
        assert metadata.to_slug() == "wet_op_de_zorgtoeslag"


class TestArticle:
    """Tests for the Article dataclass."""

    def test_basic_creation(self) -> None:
        article = Article(
            number="1",
            text="Dit is de tekst van artikel 1.",
            url="https://wetten.overheid.nl/BWBR0018451/2025-01-01#Artikel1",
        )
        assert article.number == "1"
        assert article.text == "Dit is de tekst van artikel 1."
        assert (
            article.url == "https://wetten.overheid.nl/BWBR0018451/2025-01-01#Artikel1"
        )

    def test_article_with_complex_number(self) -> None:
        article = Article(
            number="2a",
            text="Tekst van artikel 2a.",
            url="https://wetten.overheid.nl/BWBR0018451/2025-01-01#Artikel2a",
        )
        assert article.number == "2a"


class TestLaw:
    """Tests for the Law dataclass."""

    def test_basic_creation(self) -> None:
        metadata = LawMetadata(
            bwb_id="BWBR0018451",
            title="Wet op de zorgtoeslag",
            regulatory_layer=RegulatoryLayer.WET,
        )
        law = Law(metadata=metadata)

        assert law.metadata == metadata
        assert law.articles == []

    def test_creation_with_articles(self) -> None:
        metadata = LawMetadata(
            bwb_id="BWBR0018451",
            title="Wet op de zorgtoeslag",
            regulatory_layer=RegulatoryLayer.WET,
        )
        articles = [
            Article(number="1", text="Artikel 1 tekst", url="http://example.com#1"),
            Article(number="2", text="Artikel 2 tekst", url="http://example.com#2"),
        ]
        law = Law(metadata=metadata, articles=articles)

        assert len(law.articles) == 2
        assert law.articles[0].number == "1"
        assert law.articles[1].number == "2"

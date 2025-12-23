"""Data models for the harvester."""

import re

from dataclasses import dataclass, field
from enum import Enum


class RegulatoryLayer(str, Enum):
    """Types of regulatory documents."""

    WET = "WET"
    AMVB = "AMVB"
    MINISTERIELE_REGELING = "MINISTERIELE_REGELING"
    KONINKLIJK_BESLUIT = "KONINKLIJK_BESLUIT"
    BELEIDSREGEL = "BELEIDSREGEL"
    VERORDENING = "VERORDENING"
    REGELING = "REGELING"


@dataclass
class LawMetadata:
    """Metadata extracted from WTI file."""

    bwb_id: str
    title: str
    regulatory_layer: RegulatoryLayer
    publication_date: str | None = None
    effective_date: str | None = None

    def to_slug(self) -> str:
        """Generate a URL-friendly slug from the title."""

        text = self.title.lower()
        text = re.sub(r"[^\w\s-]", "", text)
        text = re.sub(r"[-\s]+", "_", text)
        return text.strip("_")


@dataclass
class Reference:
    """A reference to another article or law."""

    id: str
    bwb_id: str
    artikel: str | None = None
    lid: str | None = None
    onderdeel: str | None = None
    hoofdstuk: str | None = None
    paragraaf: str | None = None
    afdeling: str | None = None

    def to_wetten_url(self, date: str | None = None) -> str:
        """Generate wetten.overheid.nl URL.

        Args:
            date: Optional date for versioned URL (YYYY-MM-DD format)

        Returns:
            Public URL to wetten.overheid.nl
        """
        url = f"https://wetten.overheid.nl/{self.bwb_id}"
        if date:
            url += f"/{date}"

        # Build fragment for article reference
        if self.artikel:
            url += f"#Artikel{self.artikel}"

        return url

    def to_api_url(self, date: str) -> str:
        """Generate repository API URL for downloading XML.

        Args:
            date: Effective date in YYYY-MM-DD format

        Returns:
            URL to BWB repository XML file
        """
        base = "https://repository.officiele-overheidspublicaties.nl/bwb"
        return f"{base}/{self.bwb_id}/{date}_0/xml/{self.bwb_id}_{date}_0.xml"


@dataclass
class Article:
    """A single article from a law."""

    number: str
    text: str
    url: str
    references: list[Reference] = field(default_factory=list)


@dataclass
class Law:
    """Complete law with metadata and articles."""

    metadata: LawMetadata
    articles: list[Article] = field(default_factory=list)

"""Data models for the harvester."""

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
        import re

        text = self.title.lower()
        text = re.sub(r"[^\w\s-]", "", text)
        text = re.sub(r"[-\s]+", "_", text)
        return text.strip("_")


@dataclass
class Article:
    """A single article from a law."""

    number: str
    text: str
    url: str


@dataclass
class Law:
    """Complete law with metadata and articles."""

    metadata: LawMetadata
    articles: list[Article] = field(default_factory=list)
    uuid: str | None = None

    def __post_init__(self) -> None:
        """Generate UUID if not provided."""
        if self.uuid is None:
            import uuid

            self.uuid = str(uuid.uuid4())

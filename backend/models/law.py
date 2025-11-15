"""
Pydantic models for Dutch Law Schema v0.2.0

Simplified models that flexibly handle the YAML structure.
"""

from typing import Any, Optional
from datetime import date
from pydantic import BaseModel, Field, UUID4
from enum import Enum


class RegulatoryLayer(str, Enum):
    """Type of legal instrument (regelgevingslaag)"""
    GRONDWET = "GRONDWET"
    WET = "WET"
    AMVB = "AMVB"
    MINISTERIELE_REGELING = "MINISTERIELE_REGELING"
    BELEIDSREGEL = "BELEIDSREGEL"
    EU_VERORDENING = "EU_VERORDENING"
    EU_RICHTLIJN = "EU_RICHTLIJN"
    VERDRAG = "VERDRAG"
    UITVOERINGSBELEID = "UITVOERINGSBELEID"
    GEMEENTELIJKE_VERORDENING = "GEMEENTELIJKE_VERORDENING"
    PROVINCIALE_VERORDENING = "PROVINCIALE_VERORDENING"


class MachineReadable(BaseModel):
    """
    Machine-readable interpretation of an article.
    Uses flexible typing to accept the YAML structure as-is.
    """
    public: Optional[bool] = None
    endpoint: Optional[str] = None
    competent_authority: Optional[str] = None
    requires: Optional[list[dict[str, Any]]] = None
    definitions: Optional[dict[str, Any]] = None
    execution: Optional[dict[str, Any]] = None  # Flexible execution structure

    class Config:
        extra = "allow"  # Allow extra fields


class Article(BaseModel):
    """Article within a law/regulation"""
    number: str
    text: str
    url: str
    machine_readable: Optional[MachineReadable] = None

    class Config:
        extra = "allow"


class Law(BaseModel):
    """Complete law or regulation document"""
    # Schema metadata (allow but don't require)
    schema_: Optional[str] = Field(None, alias="$schema")
    id_: Optional[str] = Field(None, alias="$id")

    # Required fields
    uuid: UUID4
    publication_date: str = Field(..., pattern=r"^\d{4}-\d{2}-\d{2}$")
    regulatory_layer: RegulatoryLayer
    url: str
    articles: list[Article]

    # Optional identifiers
    bwb_id: Optional[str] = Field(None, pattern=r"^BWBR\d{7}$")
    celex_nummer: Optional[str] = None
    eli: Optional[str] = None
    tractatenblad_id: Optional[str] = None
    unts_nummer: Optional[str] = None
    gemeente_code: Optional[str] = Field(None, pattern=r"^GM\d{4}$")
    provincie_code: Optional[str] = Field(None, pattern=r"^PV\d{2}$")
    officiele_titel: Optional[str] = None
    stcrt_id: Optional[str] = None
    organisation: Optional[str] = None

    class Config:
        extra = "allow"  # Allow extra fields from YAML
        populate_by_name = True  # Allow both $id and id_


class LawSummary(BaseModel):
    """Summary of a law for list endpoints"""
    uuid: UUID4
    publication_date: str
    regulatory_layer: RegulatoryLayer
    url: str
    bwb_id: Optional[str] = None
    officiele_titel: Optional[str] = None
    article_count: int

    @classmethod
    def from_law(cls, law: Law) -> "LawSummary":
        """Create summary from full law"""
        return cls(
            uuid=law.uuid,
            publication_date=law.publication_date,
            regulatory_layer=law.regulatory_layer,
            url=law.url,
            bwb_id=law.bwb_id,
            officiele_titel=law.officiele_titel,
            article_count=len(law.articles)
        )


class ArticleWithId(Article):
    """Article with additional ID for API responses"""
    id: str  # Will be generated as "{law_uuid}:{article_number}"
    law_uuid: UUID4

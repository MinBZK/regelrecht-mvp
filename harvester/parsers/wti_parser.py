"""Parser for WTI (Wetstechnische Informatie) metadata files."""

import requests
from lxml import etree

from harvester.config import BWB_REPOSITORY_URL, HTTP_TIMEOUT
from harvester.models import LawMetadata, RegulatoryLayer


def download_wti(bwb_id: str) -> etree._Element:
    """Download WTI (metadata) file for a law.

    Args:
        bwb_id: The BWB identifier (e.g., "BWBR0018451")

    Returns:
        Parsed XML element tree

    Raises:
        requests.HTTPError: If download fails
    """
    url = f"{BWB_REPOSITORY_URL}/{bwb_id}/{bwb_id}.WTI"
    response = requests.get(url, timeout=HTTP_TIMEOUT)
    try:
        response.raise_for_status()
    except requests.HTTPError as e:
        raise requests.HTTPError(
            f"Failed to download WTI metadata for {bwb_id}: {e}"
        ) from e
    return etree.fromstring(response.content)


def parse_wti_metadata(wti_tree: etree._Element) -> LawMetadata:
    """Extract metadata from WTI XML.

    Args:
        wti_tree: Parsed WTI XML element

    Returns:
        LawMetadata with extracted fields
    """
    # BWB ID from attribute
    bwb_id = wti_tree.get("bwb-id", "")

    # Title - prefer citeertitel with status="officieel"
    title = ""
    citeertitel = wti_tree.find(".//citeertitel[@status='officieel']")
    if citeertitel is not None and citeertitel.text:
        title = citeertitel.text.strip()
    else:
        # Fallback to any citeertitel
        citeertitel = wti_tree.find(".//citeertitel")
        if citeertitel is not None and citeertitel.text:
            title = citeertitel.text.strip()

    # Regulatory layer from soort-regeling
    regulatory_layer = RegulatoryLayer.WET  # default
    soort = wti_tree.find(".//soort-regeling")
    if soort is not None and soort.text:
        soort_text = soort.text.lower()
        type_mapping = {
            "wet": RegulatoryLayer.WET,
            "amvb": RegulatoryLayer.AMVB,
            "algemene maatregel van bestuur": RegulatoryLayer.AMVB,
            "ministeriele regeling": RegulatoryLayer.MINISTERIELE_REGELING,
            "ministeriële regeling": RegulatoryLayer.MINISTERIELE_REGELING,
            "koninklijk besluit": RegulatoryLayer.KONINKLIJK_BESLUIT,
            "kb": RegulatoryLayer.KONINKLIJK_BESLUIT,
            "beleidsregel": RegulatoryLayer.BELEIDSREGEL,
            "verordening": RegulatoryLayer.VERORDENING,
            "regeling": RegulatoryLayer.REGELING,
        }
        regulatory_layer = type_mapping.get(
            soort_text,
            RegulatoryLayer.WET,
        )

    # Publication date
    publication_date = None
    pub_date_elem = wti_tree.find(".//publicatiedatum")
    if pub_date_elem is not None and pub_date_elem.text:
        publication_date = pub_date_elem.text

    return LawMetadata(
        bwb_id=bwb_id,
        title=title,
        regulatory_layer=regulatory_layer,
        publication_date=publication_date,
    )

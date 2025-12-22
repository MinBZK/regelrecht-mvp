"""Parser for JCI references in intref and extref XML elements."""

import re

from lxml import etree

from harvester.models import Reference


def parse_jci(jci_ref: str, ref_id: str) -> Reference:
    """Parse JCI reference string to Reference object.

    Args:
        jci_ref: JCI reference like "jci1.3:c:BWBR0018451&artikel=4&lid=2"
        ref_id: Unique identifier for this reference

    Returns:
        Reference object with parsed components

    Raises:
        ValueError: If no BWB ID found in JCI reference
    """
    # Extract BWB ID (required)
    bwb_match = re.search(r"(BWBR\d{7})", jci_ref)
    if not bwb_match:
        raise ValueError(f"No BWB ID found in JCI reference: {jci_ref}")

    bwb_id = bwb_match.group(1)

    # Extract optional components
    def extract_param(param: str) -> str | None:
        match = re.search(rf"{param}=([^&]+)", jci_ref)
        return match.group(1) if match else None

    return Reference(
        id=ref_id,
        bwb_id=bwb_id,
        artikel=extract_param("artikel"),
        lid=extract_param("lid"),
        onderdeel=extract_param("onderdeel"),
        hoofdstuk=extract_param("hoofdstuk"),
        paragraaf=extract_param("paragraaf"),
        afdeling=extract_param("afdeling"),
    )


def parse_intref(elem: etree._Element, ref_id: str) -> Reference | None:
    """Parse intref XML element to Reference object.

    Args:
        elem: The <intref> XML element
        ref_id: Unique identifier for this reference

    Returns:
        Reference object, or None if no valid reference found
    """
    # Try to get JCI from doc attribute
    jci_ref = elem.get("doc", "")
    if jci_ref:
        try:
            return parse_jci(jci_ref, ref_id)
        except ValueError:
            pass

    # Fallback: try to get bwb-id directly from attribute
    bwb_id = elem.get("bwb-id")
    if bwb_id:
        return Reference(id=ref_id, bwb_id=bwb_id)

    return None


def parse_extref(elem: etree._Element, ref_id: str) -> Reference | None:
    """Parse extref XML element to Reference object.

    Args:
        elem: The <extref> XML element
        ref_id: Unique identifier for this reference

    Returns:
        Reference object, or None if no valid reference found
    """
    # Same logic as intref - they have the same attributes
    return parse_intref(elem, ref_id)


def get_reference_text(elem: etree._Element) -> str:
    """Extract the display text from a reference element.

    Args:
        elem: The <intref> or <extref> XML element

    Returns:
        The text content of the element
    """
    return elem.text or ""

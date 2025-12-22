"""Text extraction utilities for parsing Dutch law XML.

This module provides low-level text extraction functions for extracting
text content from XML elements, handling inline formatting, lists, and
structural elements like lid and artikel.
"""

from __future__ import annotations

from lxml import etree

from harvester.parsers.content_parser import (
    SKIP_TAGS,
    ReferenceCollector,
    format_extref,
    format_intref,
    format_nadruk,
    get_tag_name,
)

__all__ = [
    "extract_inline_text",
    "extract_li_text",
    "get_intro_text",
    "get_li_nr",
    "get_lid_nr",
    "has_lijst",
]


def extract_inline_text(
    elem: etree._Element,
    collector: ReferenceCollector | None = None,
) -> str:
    """Extract inline text from an element without recursing into structural children.

    This extracts text content including extref/intref links and nadruk emphasis,
    but stops at structural elements like lid, lijst, li.

    Args:
        elem: XML element to extract text from
        collector: Optional reference collector for reference-style links

    Returns:
        Extracted text with markdown formatting for links and emphasis
    """
    tag_name = get_tag_name(elem)

    if tag_name in SKIP_TAGS:
        return ""

    parts: list[str] = []

    if elem.text:
        parts.append(elem.text)

    for child in elem:
        child_tag = get_tag_name(child)

        if child_tag == "extref":
            parts.append(format_extref(child, collector))

        elif child_tag == "intref":
            parts.append(format_intref(child, collector))

        elif child_tag == "nadruk":
            parts.append(format_nadruk(child))

        elif child_tag not in SKIP_TAGS and child_tag not in {
            "lid",
            "lijst",
            "li",
            "lidnr",
            "li.nr",
        }:
            # Recurse into other inline elements
            child_text = extract_inline_text(child, collector)
            if child_text:
                parts.append(child_text)

        if child.tail:
            parts.append(child.tail)

    return "".join(parts).strip()


def extract_li_text(
    li_elem: etree._Element,
    collector: ReferenceCollector | None = None,
) -> str:
    """Extract text from a list item, handling nested al elements.

    Args:
        li_elem: The <li> element
        collector: Optional reference collector for reference-style links

    Returns:
        Combined text from all <al> children
    """
    parts: list[str] = []

    for child in li_elem:
        child_tag = get_tag_name(child)
        if child_tag == "al":
            al_text = extract_inline_text(child, collector)
            if al_text:
                parts.append(al_text)

    return " ".join(parts)


def get_li_nr(li_elem: etree._Element) -> str:
    """Get the list item number (a., b., 1., etc.) from a <li> element."""
    li_nr_elem = li_elem.find(".//li.nr")
    if li_nr_elem is not None and li_nr_elem.text:
        # Remove trailing period if present
        nr = li_nr_elem.text.strip()
        if nr.endswith("."):
            nr = nr[:-1]
        return nr
    return ""


def get_lid_nr(lid_elem: etree._Element) -> str:
    """Get the lid number from a <lid> element."""
    lidnr_elem = lid_elem.find(".//lidnr")
    if lidnr_elem is not None and lidnr_elem.text:
        return lidnr_elem.text.strip()
    return ""


def has_lijst(elem: etree._Element) -> bool:
    """Check if element contains a lijst (list)."""
    return elem.find(".//lijst") is not None


def get_intro_text(
    lid_elem: etree._Element,
    collector: ReferenceCollector | None = None,
) -> str:
    """Get intro text before the lijst in a lid.

    This is text like "In deze wet wordt verstaan onder:" that appears
    before the list of onderdelen.
    """
    # Look for <al> elements that come before <lijst>
    intro_parts: list[str] = []

    for child in lid_elem:
        child_tag = get_tag_name(child)

        if child_tag == "lijst":
            # Stop when we hit the list
            break

        if child_tag == "al":
            al_text = extract_inline_text(child, collector)
            if al_text:
                intro_parts.append(al_text)

        # Skip lidnr and meta-data
        elif child_tag not in {"lidnr", "meta-data"}:
            child_text = extract_inline_text(child, collector)
            if child_text:
                intro_parts.append(child_text)

    return " ".join(intro_parts).strip()

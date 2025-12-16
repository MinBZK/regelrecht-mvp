"""Parser for consolidated legal text (content) files."""

import re

import requests
from lxml import etree

from harvester.models import Article

# Base URL for BWB repository
BWB_REPOSITORY_URL = "https://repository.officiele-overheidspublicaties.nl/bwb"

# Tags to skip when extracting text (contain metadata, not content)
SKIP_TAGS = {"meta-data", "kop", "jcis", "jci", "brondata"}


def download_content(bwb_id: str, date: str) -> etree._Element:
    """Download consolidated legal text file for a specific date.

    Uses the consolidated version URL pattern with _0 suffix to get the
    most recent version of the law as of the specified date.

    Args:
        bwb_id: The BWB identifier (e.g., "BWBR0018451")
        date: The effective date in YYYY-MM-DD format

    Returns:
        Parsed XML element tree

    Raises:
        requests.HTTPError: If download fails
    """
    # Use _0 suffix for consolidated (geconsolideerde) version
    url = f"{BWB_REPOSITORY_URL}/{bwb_id}/{date}_0/xml/{bwb_id}_{date}_0.xml"
    response = requests.get(url, timeout=30, allow_redirects=True)
    response.raise_for_status()
    return etree.fromstring(response.content)


def convert_jci_to_url(jci_ref: str) -> str:
    """Convert JCI reference to wetten.overheid.nl URL.

    Args:
        jci_ref: JCI reference like "jci1.3:c:BWBR0018450&artikel=1"

    Returns:
        URL like "https://wetten.overheid.nl/BWBR0018450#Artikel1"
    """
    # Extract BWB ID and artikel from JCI reference
    # Format: jci1.3:c:BWBR0018450&artikel=1 or jci1.3:c:BWBR0018450&artikel=1&lid=2
    bwb_match = re.search(r"BWBR\d+", jci_ref)
    artikel_match = re.search(r"artikel=(\d+\w*)", jci_ref)

    if bwb_match:
        bwb_id = bwb_match.group(0)
        if artikel_match:
            artikel = artikel_match.group(1)
            return f"https://wetten.overheid.nl/{bwb_id}#Artikel{artikel}"
        return f"https://wetten.overheid.nl/{bwb_id}"

    # Return original if can't parse
    return jci_ref


def get_tag_name(elem: etree._Element) -> str:
    """Get tag name without namespace."""
    return elem.tag.split("}")[-1] if "}" in elem.tag else elem.tag


def extract_text_from_element(elem: etree._Element | None, depth: int = 0) -> str:
    """Extract text from XML element, preserving structure as markdown.

    Args:
        elem: XML element to extract text from
        depth: Current nesting depth for indentation

    Returns:
        Extracted text with markdown formatting
    """
    if elem is None:
        return ""

    tag_name = get_tag_name(elem)

    # Skip metadata and header elements
    if tag_name in SKIP_TAGS:
        return ""

    # Handle specific element types
    if tag_name == "lid":
        # Article paragraph (lid) - extract lidnr and format
        lidnr_elem = elem.find(".//lidnr")
        lidnr = (
            lidnr_elem.text.strip()
            if lidnr_elem is not None and lidnr_elem.text
            else ""
        )

        # Get content (skip lidnr itself)
        lid_parts: list[str] = []
        for child in elem:
            child_tag = get_tag_name(child)
            if child_tag != "lidnr":
                child_text = extract_text_from_element(child, depth)
                if child_text:
                    lid_parts.append(child_text)

        lid_content = "\n\n".join(lid_parts)
        if lidnr:
            return f"{lidnr}. {lid_content}"
        return lid_content

    elif tag_name == "lijst":
        # List - process each li item
        list_items: list[str] = []
        for li in elem.findall(".//li"):
            li_text = extract_text_from_element(li, depth + 1)
            if li_text:
                list_items.append(li_text)
        return "\n".join(list_items)

    elif tag_name == "li":
        # List item - get li.nr and content
        li_nr_elem = elem.find(".//li.nr")
        li_nr = (
            li_nr_elem.text.strip()
            if li_nr_elem is not None and li_nr_elem.text
            else "-"
        )

        # Get al content within li
        li_parts: list[str] = []
        for child in elem:
            child_tag = get_tag_name(child)
            if child_tag == "al":
                al_text = extract_text_from_element(child, depth)
                if al_text:
                    li_parts.append(al_text)

        li_content = " ".join(li_parts)
        return f"- {li_nr} {li_content}"

    elif tag_name == "al":
        # Paragraph - extract inline content
        parts: list[str] = []
        if elem.text:
            parts.append(elem.text)

        for child in elem:
            child_tag = get_tag_name(child)

            if child_tag == "extref":
                # External reference - convert to markdown link
                ref_text = child.text or ""
                ref_url = child.get("doc", "")
                if ref_url:
                    converted_url = convert_jci_to_url(ref_url)
                    parts.append(f"[{ref_text}]({converted_url})")
                else:
                    parts.append(ref_text)

            elif child_tag == "intref":
                # Internal reference - convert to markdown link
                ref_text = child.text or ""
                ref_url = child.get("doc", "")
                if ref_url:
                    converted_url = convert_jci_to_url(ref_url)
                    parts.append(f"[{ref_text}]({converted_url})")
                else:
                    parts.append(ref_text)

            elif child_tag == "nadruk":
                # Emphasis
                child_text = child.text or ""
                nadruk_type = child.get("type", "")
                if nadruk_type == "vet":
                    parts.append(f"**{child_text}**")
                else:
                    parts.append(f"*{child_text}*")

            elif child_tag not in SKIP_TAGS:
                # Other inline elements
                child_text = extract_text_from_element(child, depth)
                if child_text:
                    parts.append(child_text)

            # Get tail text after child element
            if child.tail:
                parts.append(child.tail)

        return "".join(parts)

    elif tag_name == "nadruk":
        # Emphasis (standalone)
        child_text = elem.text or ""
        nadruk_type = elem.get("type", "")
        if nadruk_type == "vet":
            return f"**{child_text}**"
        return f"*{child_text}*"

    elif tag_name == "extref":
        # External reference (standalone)
        ref_text = elem.text or ""
        ref_url = elem.get("doc", "")
        if ref_url:
            converted_url = convert_jci_to_url(ref_url)
            return f"[{ref_text}]({converted_url})"
        return ref_text

    elif tag_name == "intref":
        # Internal reference (standalone)
        ref_text = elem.text or ""
        ref_url = elem.get("doc", "")
        if ref_url:
            converted_url = convert_jci_to_url(ref_url)
            return f"[{ref_text}]({converted_url})"
        return ref_text

    else:
        # Generic element - process children
        parts: list[str] = []
        if elem.text and elem.text.strip():
            parts.append(elem.text.strip())

        for child in elem:
            child_text = extract_text_from_element(child, depth)
            if child_text:
                parts.append(child_text)

            if child.tail and child.tail.strip():
                parts.append(child.tail.strip())

        return "\n\n".join(parts) if parts else ""


def parse_articles_split(
    content_tree: etree._Element,
    bwb_id: str,
    date: str,
) -> list[Article]:
    """Parse articles, splitting to lowest-level components.

    This function splits articles into their constituent parts (leden,
    onderdelen) using dot-notation numbering. For example, artikel 1,
    lid 1, onderdeel a becomes "1.1.a".

    Args:
        content_tree: Parsed content XML element
        bwb_id: The BWB identifier
        date: The effective date in YYYY-MM-DD format

    Returns:
        List of Article objects, one per lowest-level component
    """
    from harvester.parsers.article_builder import build_articles_from_content

    return build_articles_from_content(content_tree, bwb_id, date)


def parse_articles(
    content_tree: etree._Element,
    bwb_id: str,
    date: str,
) -> list[Article]:
    """Extract articles from content XML.

    Args:
        content_tree: Parsed content XML element
        bwb_id: The BWB identifier
        date: The effective date in YYYY-MM-DD format

    Returns:
        List of Article objects
    """
    articles: list[Article] = []

    # Find all artikel elements (no namespace prefix needed)
    artikel_elements = content_tree.findall(".//artikel")

    for artikel in artikel_elements:
        # Get article number from nr element (preferred) or label attribute
        article_number = ""
        nr_elem = artikel.find(".//kop/nr")
        if nr_elem is not None and nr_elem.text:
            article_number = nr_elem.text.strip()
        elif artikel.get("label"):
            # Fallback: extract number from label like "Artikel 1"
            label = artikel.get("label", "")
            if label.startswith("Artikel "):
                article_number = label.replace("Artikel ", "").strip()
            else:
                article_number = label

        if not article_number:
            continue

        # Extract text content (excluding kop and meta-data)
        content_parts: list[str] = []
        for child in artikel:
            child_tag = get_tag_name(child)
            if child_tag not in SKIP_TAGS:
                child_text = extract_text_from_element(child)
                if child_text:
                    content_parts.append(child_text)

        article_text = "\n\n".join(content_parts)

        # Generate URL
        article_url = (
            f"https://wetten.overheid.nl/{bwb_id}/{date}#Artikel{article_number}"
        )

        articles.append(
            Article(
                number=article_number,
                text=article_text,
                url=article_url,
            )
        )

    return articles

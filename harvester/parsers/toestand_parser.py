"""Parser for Toestand (consolidated legal text) files."""

import requests
from lxml import etree

from harvester.models import Article

# Base URL for BWB repository
BWB_REPOSITORY_URL = "https://repository.officiele-overheidspublicaties.nl/bwb"


def download_toestand(bwb_id: str, date: str) -> etree._Element:
    """Download Toestand (legal text) file for a specific date.

    Args:
        bwb_id: The BWB identifier (e.g., "BWBR0018451")
        date: The effective date in YYYY-MM-DD format

    Returns:
        Parsed XML element tree

    Raises:
        requests.HTTPError: If download fails
    """
    url = f"{BWB_REPOSITORY_URL}/{bwb_id}/{date}/xml/{bwb_id}_{date}.xml"
    response = requests.get(url, timeout=30, allow_redirects=True)
    response.raise_for_status()
    return etree.fromstring(response.content)


def extract_text_from_element(elem: etree._Element | None) -> str:
    """Extract text from XML element, preserving structure as markdown.

    Args:
        elem: XML element to extract text from

    Returns:
        Extracted text with markdown formatting
    """
    if elem is None:
        return ""

    text_parts: list[str] = []

    # Get direct text
    if elem.text:
        text_parts.append(elem.text.strip())

    # Process child elements
    for child in elem:
        tag_name = child.tag.split("}")[-1] if "}" in child.tag else child.tag

        if tag_name == "al":  # Paragraph
            child_text = extract_text_from_element(child)
            if child_text:
                text_parts.append(child_text)

        elif tag_name == "lijst":  # List
            for li in child.findall(".//li"):
                li_text = extract_text_from_element(li)
                if li_text:
                    text_parts.append(f"- {li_text}")

        elif tag_name == "nadruk":  # Emphasis
            child_text = extract_text_from_element(child)
            nadruk_type = child.get("type", "")
            if nadruk_type == "vet":
                text_parts.append(f"**{child_text}**")
            else:
                text_parts.append(f"*{child_text}*")

        elif tag_name == "extref":  # External reference
            ref_text = child.text or ""
            ref_url = child.get("doc", "")
            if ref_url:
                text_parts.append(f"[{ref_text}]({ref_url})")
            else:
                text_parts.append(ref_text)

        else:
            # Recursive for other elements
            child_text = extract_text_from_element(child)
            if child_text:
                text_parts.append(child_text)

        # Get tail text
        if child.tail:
            text_parts.append(child.tail.strip())

    return " ".join(part for part in text_parts if part)


def parse_articles(
    toestand_tree: etree._Element,
    bwb_id: str,
    date: str,
) -> list[Article]:
    """Extract articles from Toestand XML.

    Args:
        toestand_tree: Parsed Toestand XML element
        bwb_id: The BWB identifier
        date: The effective date in YYYY-MM-DD format

    Returns:
        List of Article objects
    """
    articles: list[Article] = []

    # Find all artikel elements (no namespace prefix needed)
    artikel_elements = toestand_tree.findall(".//artikel")

    for artikel in artikel_elements:
        # Get article number from nr element (preferred) or label attribute
        article_number = ""
        nr_elem = artikel.find(".//nr")
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

        # Extract all text from this article
        article_text = extract_text_from_element(artikel)

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

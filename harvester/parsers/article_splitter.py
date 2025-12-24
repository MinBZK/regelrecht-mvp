"""Article splitter for splitting laws into lowest-level components.

This module walks the XML tree of a Dutch law and extracts the lowest-level
components (onderdelen, leden, or full articles) as separate Article objects
with dot-notation numbering (e.g., 1.1.a for artikel 1, lid 1, onderdeel a).
"""

from __future__ import annotations

from lxml import etree

from harvester.models import Article
from harvester.parsers.content_parser import ReferenceCollector
from harvester.parsers.splitting import (
    ArticleComponent,
    LeafSplitStrategy,
    SplitContext,
    SplitEngine,
    create_dutch_law_hierarchy,
)
from harvester.parsers.text_extractor import extract_inline_text

# Module-level instances (created once, reused)
_hierarchy = create_dutch_law_hierarchy()
_strategy = LeafSplitStrategy()
_engine = SplitEngine(_hierarchy, _strategy)

# Re-export ArticleComponent for backwards compatibility
__all__ = ["ArticleComponent", "build_articles_from_content", "extract_aanhef"]


def extract_aanhef(
    content_tree: etree._Element,
    bwb_id: str,
    date: str,
) -> Article | None:
    """Extract the aanhef (preamble) as an article.

    The aanhef contains the royal introduction ("Wij Beatrix..."),
    considerans (considerations), and afkondiging (proclamation).

    Args:
        content_tree: Parsed content XML element
        bwb_id: BWB identifier
        date: Effective date in YYYY-MM-DD format

    Returns:
        Article with number "aanhef", or None if no aanhef found
    """
    aanhef_elem = content_tree.find(".//aanhef")
    if aanhef_elem is None:
        return None

    # Create collector for aanhef
    collector = ReferenceCollector()
    parts: list[str] = []

    # Extract <wij> element
    wij_elem = aanhef_elem.find("wij")
    if wij_elem is not None and wij_elem.text:
        parts.append(wij_elem.text.strip())

    # Extract <considerans> elements
    considerans_elem = aanhef_elem.find("considerans")
    if considerans_elem is not None:
        for al in considerans_elem.findall(".//considerans.al"):
            al_text = extract_inline_text(al, collector)
            if al_text:
                parts.append(al_text)

    # Extract <afkondiging> element
    afkondiging_elem = aanhef_elem.find("afkondiging")
    if afkondiging_elem is not None:
        for al in afkondiging_elem.findall(".//al"):
            al_text = extract_inline_text(al, collector)
            if al_text:
                parts.append(al_text)

    if not parts:
        return None

    aanhef_text = "\n\n".join(parts)
    aanhef_url = f"https://wetten.overheid.nl/{bwb_id}/{date}#Aanhef"

    # Add reference definitions if any references were collected
    if collector.references:
        ref_lines = []
        for ref in collector.references:
            url = ref.to_wetten_url()
            ref_lines.append(f"[{ref.id}]: {url}")
        aanhef_text = f"{aanhef_text}\n\n" + "\n".join(ref_lines)

    return Article(
        number="aanhef",
        text=aanhef_text,
        url=aanhef_url,
        references=collector.references.copy(),
    )


def build_articles_from_content(
    content_tree: etree._Element,
    bwb_id: str,
    date: str,
) -> list[Article]:
    """Build flat list of articles from content XML, split to lowest level.

    This is the main entry point for the article splitter. It walks all
    artikel elements and extracts the lowest-level components as separate
    Article objects. The aanhef (preamble) is included as the first article
    with number "aanhef".

    Each component gets its own reference collector, so reference-style
    links work correctly with definitions included in each component.

    Args:
        content_tree: Parsed content XML element
        bwb_id: BWB identifier
        date: Effective date in YYYY-MM-DD format

    Returns:
        List of Article objects, one per lowest-level component
    """
    articles: list[Article] = []

    # Extract aanhef first
    aanhef = extract_aanhef(content_tree, bwb_id, date)
    if aanhef:
        articles.append(aanhef)

    # Then extract all artikel elements using the SplitEngine
    artikel_elements = content_tree.findall(".//artikel")

    for artikel in artikel_elements:
        # Get article number for base URL construction
        nr_elem = artikel.find(".//kop/nr")
        if nr_elem is not None and nr_elem.text:
            artikel_nr = nr_elem.text.strip()
        elif artikel.get("label"):
            label = artikel.get("label", "")
            if label.startswith("Artikel "):
                artikel_nr = label.replace("Artikel ", "").strip()
            else:
                artikel_nr = label
        else:
            continue  # Skip articles without number

        # Replace spaces with underscores in URL fragment (e.g., "A 1" -> "A_1")
        artikel_nr_url = artikel_nr.replace(" ", "_")
        base_url = f"https://wetten.overheid.nl/{bwb_id}/{date}#Artikel{artikel_nr_url}"

        # Create context for splitting
        context = SplitContext(
            bwb_id=bwb_id,
            date=date,
            base_url=base_url,
        )

        # Split the artikel using the engine
        components = _engine.split(artikel, context)

        # Convert components to articles
        for component in components:
            articles.append(component.to_article())

    return articles

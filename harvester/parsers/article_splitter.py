"""Article builder for splitting laws into lowest-level components.

This module walks the XML tree of a Dutch law and extracts the lowest-level
components (onderdelen, leden, or full articles) as separate Article objects
with dot-notation numbering (e.g., 1.1.a for artikel 1, lid 1, onderdeel a).
"""

from __future__ import annotations

from dataclasses import dataclass, field

from lxml import etree

from harvester.models import Article, Reference
from harvester.parsers.content_parser import (
    SKIP_TAGS,
    ReferenceCollector,
    get_tag_name,
)
from harvester.parsers.text_extractor import (
    extract_inline_text,
    extract_li_text,
    get_intro_text,
    get_li_nr,
    get_lid_nr,
)


@dataclass
class ArticleComponent:
    """Represents a lowest-level component of an article."""

    number_parts: list[str]  # e.g., ["1", "1", "a"] for artikel 1, lid 1, onderdeel a
    text: str
    base_url: str  # Base URL for the article (without fragment)
    references: list[Reference] = field(default_factory=list)

    def to_number(self) -> str:
        """Convert number parts to dot notation."""
        return ".".join(self.number_parts)

    def to_article(self) -> Article:
        """Convert to Article object with reference definitions appended to text."""
        text = self.text

        # Append reference definitions if there are any references
        if self.references:
            ref_lines = []
            for ref in self.references:
                url = ref.to_wetten_url()
                ref_lines.append(f"[{ref.id}]: {url}")
            text = f"{text}\n\n" + "\n".join(ref_lines)

        return Article(
            number=self.to_number(),
            text=text,
            url=self.base_url,
            references=self.references,
        )


def walk_lijst(
    lijst_elem: etree._Element,
    number_parts: list[str],
    base_url: str,
) -> list[ArticleComponent]:
    """Walk a lijst and extract components from each li.

    Handles nested lists by recursing deeper.
    For unmarked lists (type="ongemarkeerd" with dashes), uses sequential numbering.
    Each component gets its own collector for reference-style links.

    Args:
        lijst_elem: The <lijst> element
        number_parts: Current number parts (e.g., ["1", "1"])
        base_url: Base URL for article

    Returns:
        List of ArticleComponent for each lowest-level item
    """
    components: list[ArticleComponent] = []

    # Check if this is an unmarked list (uses dashes instead of numbers/letters)
    is_unmarked = lijst_elem.get("type") == "ongemarkeerd"
    seq_counter = 0

    for li in lijst_elem.findall("li"):
        li_nr = get_li_nr(li)

        # For unmarked lists with dashes, use sequential numbering
        if is_unmarked and (not li_nr or li_nr in {"\u2013", "-", "\u2014"}):
            seq_counter += 1
            li_nr = str(seq_counter)
        elif not li_nr:
            continue

        # Check for nested lijst
        nested_lijst = li.find("lijst")
        if nested_lijst is not None:
            # Get any intro text in this li before the nested list
            # Each intro gets its own collector
            intro_collector = ReferenceCollector()
            li_intro = ""
            for child in li:
                child_tag = get_tag_name(child)
                if child_tag == "lijst":
                    break
                if child_tag == "al":
                    text = extract_inline_text(child, intro_collector)
                    if text:
                        li_intro = text
                        break

            # If there's intro text, add it as a component with its references
            if li_intro:
                components.append(
                    ArticleComponent(
                        number_parts=[*number_parts, li_nr],
                        text=li_intro,
                        base_url=base_url,
                        references=intro_collector.references.copy(),
                    )
                )

            # Recurse into nested list
            components.extend(
                walk_lijst(nested_lijst, [*number_parts, li_nr], base_url)
            )
        else:
            # Leaf node - extract text with its own collector
            li_collector = ReferenceCollector()
            li_text = extract_li_text(li, li_collector)
            if li_text:
                components.append(
                    ArticleComponent(
                        number_parts=[*number_parts, li_nr],
                        text=li_text,
                        base_url=base_url,
                        references=li_collector.references.copy(),
                    )
                )

    return components


def walk_lid(
    lid_elem: etree._Element,
    artikel_nr: str,
    base_url: str,
) -> list[ArticleComponent]:
    """Walk a lid element and extract components.

    If lid contains a lijst, extracts intro text separately and then
    each list item. Otherwise extracts the whole lid as one component.
    Each component gets its own collector for reference-style links.

    Args:
        lid_elem: The <lid> element
        artikel_nr: Article number (e.g., "1")
        base_url: Base URL for article

    Returns:
        List of ArticleComponent objects
    """
    components: list[ArticleComponent] = []
    lid_nr = get_lid_nr(lid_elem)

    if not lid_nr:
        return components

    number_parts = [artikel_nr, lid_nr]

    lijst_elem = lid_elem.find("lijst")

    if lijst_elem is not None:
        # Has a list - extract intro text first with its own collector
        intro_collector = ReferenceCollector()
        intro = get_intro_text(lid_elem, intro_collector)
        if intro:
            components.append(
                ArticleComponent(
                    number_parts=number_parts.copy(),
                    text=intro,
                    base_url=base_url,
                    references=intro_collector.references.copy(),
                )
            )

        # Then walk the list (each item gets its own collector inside walk_lijst)
        components.extend(walk_lijst(lijst_elem, number_parts, base_url))
    else:
        # No list - extract all text from lid with one collector
        lid_collector = ReferenceCollector()
        lid_parts: list[str] = []
        for child in lid_elem:
            child_tag = get_tag_name(child)
            if child_tag == "al":
                al_text = extract_inline_text(child, lid_collector)
                if al_text:
                    lid_parts.append(al_text)
            elif child_tag not in {"lidnr", "meta-data"}:
                child_text = extract_inline_text(child, lid_collector)
                if child_text:
                    lid_parts.append(child_text)

        lid_text = " ".join(lid_parts).strip()
        if lid_text:
            components.append(
                ArticleComponent(
                    number_parts=number_parts,
                    text=lid_text,
                    base_url=base_url,
                    references=lid_collector.references.copy(),
                )
            )

    return components


def walk_artikel(
    artikel_elem: etree._Element,
    bwb_id: str,
    date: str,
) -> list[ArticleComponent]:
    """Walk an artikel element and extract all lowest-level components.

    Each component gets its own collector for reference-style links.

    Args:
        artikel_elem: The <artikel> element
        bwb_id: BWB identifier
        date: Effective date

    Returns:
        List of ArticleComponent objects
    """
    components: list[ArticleComponent] = []

    # Get article number
    nr_elem = artikel_elem.find(".//kop/nr")
    if nr_elem is not None and nr_elem.text:
        artikel_nr = nr_elem.text.strip()
    elif artikel_elem.get("label"):
        label = artikel_elem.get("label", "")
        if label.startswith("Artikel "):
            artikel_nr = label.replace("Artikel ", "").strip()
        else:
            artikel_nr = label
    else:
        return components  # Skip articles without number

    # Replace spaces with underscores in URL fragment (e.g., "A 1" -> "A_1")
    artikel_nr_url = artikel_nr.replace(" ", "_")
    base_url = f"https://wetten.overheid.nl/{bwb_id}/{date}#Artikel{artikel_nr_url}"

    # Check if artikel has leden
    leden = artikel_elem.findall("lid")

    if leden:
        # Has leden - walk each one (each component gets its own collector inside)
        for lid in leden:
            components.extend(walk_lid(lid, artikel_nr, base_url))
    else:
        # No leden - check for direct lijst
        direct_lijst = artikel_elem.find("lijst")

        if direct_lijst is not None:
            # Artikel has direct list (e.g., definition lists without leden)
            # Get intro text before the list with its own collector
            intro_collector = ReferenceCollector()
            intro_parts: list[str] = []
            for child in artikel_elem:
                child_tag = get_tag_name(child)
                if child_tag == "lijst":
                    break
                if child_tag == "al":
                    al_text = extract_inline_text(child, intro_collector)
                    if al_text:
                        intro_parts.append(al_text)

            intro_text = " ".join(intro_parts).strip()
            if intro_text:
                components.append(
                    ArticleComponent(
                        number_parts=[artikel_nr],
                        text=intro_text,
                        base_url=base_url,
                        references=intro_collector.references.copy(),
                    )
                )

            # Walk the list
            components.extend(walk_lijst(direct_lijst, [artikel_nr], base_url))
        else:
            # No leden and no list - treat whole article as single component
            artikel_collector = ReferenceCollector()
            artikel_parts: list[str] = []
            for child in artikel_elem:
                child_tag = get_tag_name(child)
                if child_tag == "al":
                    al_text = extract_inline_text(child, artikel_collector)
                    if al_text:
                        artikel_parts.append(al_text)
                elif child_tag not in SKIP_TAGS:
                    child_text = extract_inline_text(child, artikel_collector)
                    if child_text:
                        artikel_parts.append(child_text)

            artikel_text = " ".join(artikel_parts).strip()
            if artikel_text:
                components.append(
                    ArticleComponent(
                        number_parts=[artikel_nr],
                        text=artikel_text,
                        base_url=base_url,
                        references=artikel_collector.references.copy(),
                    )
                )

    return components


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
            if al.text:
                parts.append(al.text.strip())

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

    This is the main entry point for the article builder. It walks all
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

    # Then extract all artikel elements
    artikel_elements = content_tree.findall(".//artikel")

    for artikel in artikel_elements:
        components = walk_artikel(artikel, bwb_id, date)
        for component in components:
            articles.append(component.to_article())

    return articles

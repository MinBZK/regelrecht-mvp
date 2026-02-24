#!/usr/bin/env python3
"""
Convert CVDR XML files to regelrecht YAML format.

Usage:
    uv run python script/convert_cvdr.py /tmp/cvdr701597.xml
"""

import sys
import uuid
import re
from pathlib import Path
import yaml
from lxml import etree

# XML Namespaces for CVDR
CVDR_NS = {
    "cvdr": "http://standaarden.overheid.nl/cvdr/terms/",
    "dcterms": "http://purl.org/dc/terms/",
    "overheid": "http://standaarden.overheid.nl/owms/terms/",
    "overheidrg": "http://standaarden.overheid.nl/cvdr/meta/",
}


def slugify(text: str) -> str:
    """Convert text to URL-friendly slug."""
    text = text.lower()
    text = re.sub(r"[^\w\s-]", "", text)
    text = re.sub(r"[-\s]+", "_", text)
    return text.strip("_")


def extract_text_content(elem) -> str:
    """Extract text content from element, handling nested elements."""
    if elem is None:
        return ""

    parts = []
    if elem.text:
        parts.append(elem.text)

    for child in elem:
        tag = child.tag.split("}")[-1] if "}" in child.tag else child.tag

        if tag == "vet":
            parts.append(f"**{extract_text_content(child)}**")
        elif tag == "cursief":
            parts.append(f"*{extract_text_content(child)}*")
        elif tag == "onderstreept":
            parts.append(extract_text_content(child))
        elif tag == "extref":
            url = child.get("doc", "")
            text = extract_text_content(child)
            parts.append(f"[{text}]({url})")
        elif tag in ("al", "li", "lid"):
            parts.append(extract_text_content(child))
        else:
            parts.append(extract_text_content(child))

        if child.tail:
            parts.append(child.tail)

    return "".join(parts)


def parse_artikel_ns(artikel_elem) -> dict:
    """Parse a single artikel element with CVDR namespace."""
    article = {}

    # Get article number and title
    kop = artikel_elem.find("cvdr:kop", namespaces=CVDR_NS)
    if kop is not None:
        nr = kop.find("cvdr:nr", namespaces=CVDR_NS)
        if nr is not None:
            article["number"] = nr.text
        titel = kop.find("cvdr:titel", namespaces=CVDR_NS)
        if titel is not None:
            article["title"] = titel.text

    # Get article text
    text_parts = []

    # Add title if present
    if "title" in article:
        text_parts.append(f"**{article['title']}**\n\n")

    # Process lids (paragraphs)
    for lid in artikel_elem.findall(".//cvdr:lid", namespaces=CVDR_NS):
        lidnr = lid.find("cvdr:lidnr", namespaces=CVDR_NS)
        if lidnr is not None and lidnr.text:
            text_parts.append(f"{lidnr.text} ")

        for al in lid.findall("cvdr:al", namespaces=CVDR_NS):
            text_parts.append(extract_text_content(al))
            text_parts.append("\n")

        # Handle lists within lid
        for lijst in lid.findall("cvdr:lijst", namespaces=CVDR_NS):
            for li in lijst.findall("cvdr:li", namespaces=CVDR_NS):
                nr_attr = li.get("nr", "-")
                for al in li.findall("cvdr:al", namespaces=CVDR_NS):
                    text_parts.append(f"  {nr_attr} {extract_text_content(al)}\n")

    # Process direct al elements (not in lid)
    for al in artikel_elem.findall("cvdr:al", namespaces=CVDR_NS):
        text_parts.append(extract_text_content(al))
        text_parts.append("\n")

    # Process lists not in lid
    for lijst in artikel_elem.findall("cvdr:lijst", namespaces=CVDR_NS):
        for li in lijst.findall("cvdr:li", namespaces=CVDR_NS):
            nr_attr = li.get("nr", "-")
            for al in li.findall("cvdr:al", namespaces=CVDR_NS):
                text_parts.append(f"  {nr_attr} {extract_text_content(al)}\n")
            # Handle nested lists
            for nested_lijst in li.findall("cvdr:lijst", namespaces=CVDR_NS):
                for nested_li in nested_lijst.findall("cvdr:li", namespaces=CVDR_NS):
                    nested_nr = nested_li.get("nr", "-")
                    for nested_al in nested_li.findall("cvdr:al", namespaces=CVDR_NS):
                        text_parts.append(f"    {nested_nr} {extract_text_content(nested_al)}\n")

    article["text"] = "".join(text_parts).strip()
    return article


def parse_artikel(artikel_elem) -> dict:
    """Parse a single artikel element."""
    article = {}

    # Get article number and title
    kop = artikel_elem.find(".//kop", namespaces=None)
    if kop is not None:
        nr = kop.find("nr")
        if nr is not None:
            article["number"] = nr.text
        titel = kop.find("titel")
        if titel is not None:
            article["title"] = titel.text

    # Get article text
    text_parts = []

    # Add title if present
    if "title" in article:
        text_parts.append(f"**{article['title']}**\n")

    # Process lids (paragraphs)
    for lid in artikel_elem.findall(".//lid"):
        lidnr = lid.find("lidnr")
        if lidnr is not None and lidnr.text:
            text_parts.append(f"{lidnr.text} ")

        for al in lid.findall("al"):
            text_parts.append(extract_text_content(al))
            text_parts.append("\n")

        # Handle lists within lid
        for lijst in lid.findall("lijst"):
            for li in lijst.findall("li"):
                nr_attr = li.get("nr", "-")
                for al in li.findall("al"):
                    text_parts.append(f"  {nr_attr} {extract_text_content(al)}\n")

    # Process direct al elements (not in lid)
    for al in artikel_elem.findall("al"):
        text_parts.append(extract_text_content(al))
        text_parts.append("\n")

    # Process lists not in lid
    for lijst in artikel_elem.findall("lijst"):
        for li in lijst.findall("li"):
            nr_attr = li.get("nr", "-")
            for al in li.findall("al"):
                text_parts.append(f"  {nr_attr} {extract_text_content(al)}\n")
            # Handle nested lists
            for nested_lijst in li.findall("lijst"):
                for nested_li in nested_lijst.findall("li"):
                    nested_nr = nested_li.get("nr", "-")
                    for nested_al in nested_li.findall("al"):
                        text_parts.append(f"    {nested_nr} {extract_text_content(nested_al)}\n")

    article["text"] = "".join(text_parts).strip()
    return article


def convert_cvdr_to_yaml(xml_path: Path) -> dict:
    """Convert CVDR XML to YAML structure."""
    tree = etree.parse(str(xml_path))
    root = tree.getroot()

    # Extract metadata
    identifier = root.find(".//dcterms:identifier", namespaces=CVDR_NS)
    title = root.find(".//dcterms:title", namespaces=CVDR_NS)
    creator = root.find(".//dcterms:creator", namespaces=CVDR_NS)
    modified = root.find(".//dcterms:modified", namespaces=CVDR_NS)
    issued = root.find(".//dcterms:issued", namespaces=CVDR_NS)
    inwerking = root.find(".//overheidrg:inwerkingtredingDatum", namespaces=CVDR_NS)

    cvdr_id = identifier.text if identifier is not None else "unknown"
    title_text = title.text if title is not None else "Unknown"
    creator_text = creator.text if creator is not None else "Unknown"

    # Determine dates
    valid_from = inwerking.text if inwerking is not None else (issued.text if issued is not None else modified.text)
    publication_date = issued.text if issued is not None else modified.text

    # Build YAML structure
    yaml_data = {
        "$schema": "https://regelrecht.nl/schema/v0.3.0/schema.json",
        "$id": slugify(title_text),
        "uuid": f"cvdr{cvdr_id.replace('CVDR', '').replace('_', '-')}",
        "regulatory_layer": "UITVOERINGSBELEID",
        "publication_date": publication_date,
        "valid_from": valid_from,
        "gemeente_code": "GM0599",  # Rotterdam
        "officiele_titel": title_text,
        "url": f"https://lokaleregelgeving.overheid.nl/{cvdr_id.split('_')[0]}",
        "name": title_text,
        "articles": []
    }

    # Extract articles - they are in the CVDR namespace
    for artikel in root.findall(".//cvdr:artikel", namespaces=CVDR_NS):
        article_data = parse_artikel_ns(artikel)
        if article_data.get("number"):
            yaml_data["articles"].append({
                "number": article_data["number"],
                "text": article_data["text"],
                "url": f"https://lokaleregelgeving.overheid.nl/{cvdr_id.split('_')[0]}#Artikel{article_data['number']}"
            })

    return yaml_data


def main():
    if len(sys.argv) < 2:
        print("Usage: uv run python script/convert_cvdr.py <xml_file>")
        sys.exit(1)

    xml_path = Path(sys.argv[1])
    if not xml_path.exists():
        print(f"File not found: {xml_path}")
        sys.exit(1)

    yaml_data = convert_cvdr_to_yaml(xml_path)

    # Determine output path
    output_dir = Path("regulation/nl/gemeentelijke_verordening/rotterdam/uitvoeringsbeleid")
    output_dir.mkdir(parents=True, exist_ok=True)

    output_file = output_dir / f"{yaml_data['$id']}_{yaml_data['valid_from']}.yaml"

    # Write YAML
    with open(output_file, "w", encoding="utf-8") as f:
        f.write("---\n")
        yaml.dump(yaml_data, f, allow_unicode=True, default_flow_style=False, sort_keys=False, width=100)

    print(f"✅ Converted to: {output_file}")
    print(f"   Title: {yaml_data['name']}")
    print(f"   Articles: {len(yaml_data['articles'])}")

    return output_file


if __name__ == "__main__":
    main()

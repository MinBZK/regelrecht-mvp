"""Regenerate expected YAML output for harvester integration tests."""

import sys
from pathlib import Path

# Add project root to path so we can import harvester
sys.path.insert(0, str(Path(__file__).parent.parent))

import ruamel.yaml
from lxml import etree

from harvester.models import Law
from harvester.parsers.content_parser import parse_articles_split
from harvester.parsers.wti_parser import parse_wti_metadata
from harvester.storage.yaml_writer import generate_yaml_dict

FIXTURES_DIR = Path(__file__).parent.parent / "tests" / "test_harvester" / "fixtures"

# Fixtures to regenerate: (name, effective_date)
FIXTURES = [
    ("zorgtoeslag", "2025-01-01"),
    ("kieswet", "2025-08-01"),
    ("wlz", "2025-07-05"),
    ("zvw", "2025-07-05"),
    ("awir", "2025-01-01"),
]


def update_fixture(name: str, date: str) -> None:
    """Update expected YAML for a single fixture."""
    wti_path = FIXTURES_DIR / f"{name}_wti.xml"
    toestand_path = FIXTURES_DIR / f"{name}_toestand.xml"

    if not wti_path.exists() or not toestand_path.exists():
        print(f"Skipping {name}: XML files not found")
        return

    wti_tree = etree.parse(str(wti_path))
    toestand_tree = etree.parse(str(toestand_path))

    metadata = parse_wti_metadata(wti_tree.getroot())
    articles = parse_articles_split(toestand_tree.getroot(), metadata.bwb_id, date)
    law = Law(metadata=metadata, articles=articles)
    yaml_dict = generate_yaml_dict(law, date)

    output_path = FIXTURES_DIR / f"{name}_expected.yaml"

    # Configure ruamel.yaml for proper formatting
    yaml = ruamel.yaml.YAML()
    yaml.default_flow_style = False
    yaml.preserve_quotes = True
    yaml.indent(mapping=2, sequence=4, offset=2)
    yaml.width = 100

    with open(output_path, "w", encoding="utf-8", newline="\n") as f:
        yaml.dump(yaml_dict, f)
    print(f"Updated {output_path} ({len(articles)} articles)")


def main() -> None:
    """Update expected YAML output from input XML fixtures."""
    for name, date in FIXTURES:
        update_fixture(name, date)


if __name__ == "__main__":
    main()

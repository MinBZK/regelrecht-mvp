"""YAML writer for law files."""

from pathlib import Path

import yaml

from harvester.models import Law

# Schema URL for regelrecht YAML files
SCHEMA_URL = "https://raw.githubusercontent.com/MinBZK/poc-machine-law/refs/heads/main/schema/v0.2.0/schema.json"


def generate_yaml_dict(law: Law, effective_date: str) -> dict:
    """Generate a schema-compliant dictionary from a Law object.

    Args:
        law: The Law object to convert
        effective_date: The effective date in YYYY-MM-DD format

    Returns:
        Dictionary ready for YAML serialization
    """
    law_id = law.metadata.to_slug()

    return {
        "$schema": SCHEMA_URL,
        "$id": law_id,
        "uuid": law.uuid,
        "regulatory_layer": law.metadata.regulatory_layer.value,
        "publication_date": law.metadata.publication_date or effective_date,
        "bwb_id": law.metadata.bwb_id,
        "url": f"https://wetten.overheid.nl/{law.metadata.bwb_id}/{effective_date}",
        "articles": [
            {
                "number": article.number,
                "text": article.text,
                "url": article.url,
            }
            for article in law.articles
        ],
    }


def save_yaml(
    law: Law,
    effective_date: str,
    output_base: Path | None = None,
) -> Path:
    """Save a Law object as a YAML file.

    Args:
        law: The Law object to save
        effective_date: The effective date in YYYY-MM-DD format
        output_base: Base directory for output (default: regulation/nl/)

    Returns:
        Path to the saved file
    """
    if output_base is None:
        output_base = Path("regulation/nl")

    # Determine directory structure
    layer_dir = law.metadata.regulatory_layer.value.lower()
    law_id = law.metadata.to_slug()
    output_dir = output_base / layer_dir / law_id
    output_dir.mkdir(parents=True, exist_ok=True)

    output_file = output_dir / f"{effective_date}.yaml"

    # Generate YAML content
    yaml_dict = generate_yaml_dict(law, effective_date)

    with open(output_file, "w", encoding="utf-8") as f:
        yaml.dump(
            yaml_dict,
            f,
            allow_unicode=True,
            sort_keys=False,
            default_flow_style=False,
            width=100,
        )

    return output_file

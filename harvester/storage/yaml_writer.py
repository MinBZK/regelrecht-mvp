"""YAML writer for law files."""

import textwrap
from pathlib import Path

import ruamel.yaml

from harvester.models import Law

# Schema URL for regelrecht YAML files
SCHEMA_URL = "https://raw.githubusercontent.com/MinBZK/poc-machine-law/refs/heads/main/schema/v0.3.0/schema.json"

TEXT_WRAP_WIDTH = 100


def _wrap_text(text: str, width: int = TEXT_WRAP_WIDTH) -> str:
    """Wrap text at specified width, preserving existing paragraph breaks."""
    paragraphs = text.split("\n\n")
    wrapped = [textwrap.fill(p, width=width) for p in paragraphs]
    return "\n\n".join(wrapped)


def _should_wrap_text(text: str) -> bool:
    """Check if text should be wrapped for readability."""
    return (
        len(text) > 80 or "[" in text and "](" in text  # Markdown links
    )


def generate_yaml_dict(law: Law, effective_date: str) -> dict:
    """Generate a schema-compliant dictionary from a Law object.

    Args:
        law: The Law object to convert
        effective_date: The effective date (inwerkingtredingsdatum) in YYYY-MM-DD format

    Returns:
        Dictionary ready for YAML serialization
    """
    law_id = law.metadata.to_slug()

    def format_article_text(text: str) -> str:
        """Wrap article text for readability if needed."""
        if _should_wrap_text(text):
            return _wrap_text(text)
        return text

    # RFC-001 Decision 7: uuid field removed (no clear purpose identified)
    return {
        "$schema": SCHEMA_URL,
        "$id": law_id,
        "regulatory_layer": law.metadata.regulatory_layer.value,
        "publication_date": law.metadata.publication_date or effective_date,
        "valid_from": effective_date,
        "bwb_id": law.metadata.bwb_id,
        "url": f"https://wetten.overheid.nl/{law.metadata.bwb_id}/{effective_date}",
        "articles": [
            {
                "number": article.number,
                "text": format_article_text(article.text),
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

    # Configure ruamel.yaml for proper formatting
    yaml = ruamel.yaml.YAML()
    yaml.default_flow_style = False
    yaml.preserve_quotes = True
    yaml.indent(mapping=2, sequence=4, offset=2)  # indent-sequences: true
    yaml.width = 100
    yaml.explicit_start = True  # Add --- document start

    # Write with Unix line endings
    with open(output_file, "w", encoding="utf-8", newline="\n") as f:
        yaml.dump(yaml_dict, f)

    return output_file

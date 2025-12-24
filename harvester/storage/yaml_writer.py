"""YAML writer for law files."""

import io
import textwrap
from pathlib import Path

import ruamel.yaml
from ruamel.yaml.scalarstring import LiteralScalarString

from harvester.models import Law, Reference

# Schema URL for regelrecht YAML files
SCHEMA_URL = "https://raw.githubusercontent.com/MinBZK/poc-machine-law/refs/heads/main/schema/v0.3.1/schema.json"

TEXT_WRAP_WIDTH = 100


def _wrap_text(text: str, width: int = TEXT_WRAP_WIDTH) -> str:
    """Wrap text at specified width, preserving paragraph breaks and reference definitions.

    Reference definitions (lines starting with [refN]:) are preserved as-is
    to maintain valid markdown reference-style links.
    """
    # Separate reference definitions from main text
    # Reference definitions are at the end, each on their own line
    lines = text.split("\n")
    ref_lines: list[str] = []
    content_lines: list[str] = []

    # Find where reference definitions start (from end)
    in_refs = False
    for line in reversed(lines):
        if line.startswith("[ref") and "]: " in line:
            ref_lines.insert(0, line)
            in_refs = True
        elif in_refs and line == "":
            # Empty line before refs
            ref_lines.insert(0, line)
        else:
            in_refs = False
            content_lines.insert(0, line)

    # Wrap content paragraphs
    content_text = "\n".join(content_lines)
    paragraphs = content_text.split("\n\n")
    wrapped = [textwrap.fill(p, width=width) for p in paragraphs]
    wrapped_content = "\n\n".join(wrapped)

    # Append reference definitions unchanged
    if ref_lines:
        return wrapped_content + "\n\n" + "\n".join(ref_lines)
    return wrapped_content


def _should_wrap_text(text: str) -> bool:
    """Check if text should be wrapped for readability."""
    has_markdown_links = "[" in text and "](" in text
    return len(text) > 80 or has_markdown_links


def _reference_to_dict(ref: Reference) -> dict:
    """Convert a Reference to a dictionary with only non-None fields.

    Args:
        ref: The Reference object

    Returns:
        Dictionary with id, bwb_id, and any optional fields that are set
    """
    result = {"id": ref.id, "bwb_id": ref.bwb_id}

    # Add optional fields if present
    if ref.artikel:
        result["artikel"] = ref.artikel
    if ref.lid:
        result["lid"] = ref.lid
    if ref.onderdeel:
        result["onderdeel"] = ref.onderdeel
    if ref.hoofdstuk:
        result["hoofdstuk"] = ref.hoofdstuk
    if ref.paragraaf:
        result["paragraaf"] = ref.paragraaf
    if ref.afdeling:
        result["afdeling"] = ref.afdeling

    return result


def generate_yaml_dict(law: Law, effective_date: str) -> dict:
    """Generate a schema-compliant dictionary from a Law object.

    Args:
        law: The Law object to convert
        effective_date: The effective date (inwerkingtredingsdatum) in YYYY-MM-DD format

    Returns:
        Dictionary ready for YAML serialization
    """
    law_id = law.metadata.to_slug()

    def format_article_text(text: str) -> str | LiteralScalarString:
        """Wrap article text for readability and use literal block scalar.

        RFC-001 Decision 2: Use |- (literal block scalar) for multiline text.
        """
        if _should_wrap_text(text):
            text = _wrap_text(text)
        # Use literal block scalar (|-) for multiline text
        if "\n" in text:
            return LiteralScalarString(text)
        return text

    def _format_article_dict(article, format_text_fn):
        """Format an article as a dictionary, including references if present."""
        result = {
            "number": article.number,
            "text": format_text_fn(article.text),
            "url": article.url,
        }
        if article.references:
            result["references"] = [
                _reference_to_dict(ref) for ref in article.references
            ]
        return result

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
            _format_article_dict(article, format_article_text)
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

    # Write to buffer first, then strip trailing spaces
    # (ruamel.yaml adds trailing spaces when wrapping long values like $schema)
    buffer = io.StringIO()
    yaml.dump(yaml_dict, buffer)
    content = buffer.getvalue()
    content = "\n".join(line.rstrip() for line in content.splitlines()) + "\n"

    # Write with Unix line endings
    with open(output_file, "w", encoding="utf-8", newline="\n") as f:
        f.write(content)

    return output_file

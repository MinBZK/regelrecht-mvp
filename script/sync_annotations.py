#!/usr/bin/env python3
"""
Annotation Sync Script

Promotes approved W3C annotations to the law YAML files.

Data flow:
1. Read annotations from annotations/{reg_id}.yaml
2. Filter annotations with status: approved
3. Convert to schema structures using AnnotationConverter
4. Validate against schema/v0.3.0/schema.json
5. Update regulation/{reg_id}/YYYY-MM-DD.yaml
6. Mark annotations as status: promoted with promoted_at timestamp

Usage:
    python script/sync_annotations.py participatiewet --preview
    python script/sync_annotations.py participatiewet --execute
    python script/sync_annotations.py --all --preview
"""

import argparse
import json
import sys
from dataclasses import dataclass
from pathlib import Path

import yaml

# Add parent directory to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent))

from script.annotation_converter import (
    AnnotationConverter,
    ConversionResult,
    apply_conversion_to_article,
    mark_annotation_promoted,
)

# Paths
PROJECT_ROOT = Path(__file__).parent.parent
REGULATION_DIR = PROJECT_ROOT / "regulation" / "nl"
ANNOTATIONS_DIR = PROJECT_ROOT / "annotations"
SCHEMA_PATH = PROJECT_ROOT / "schema" / "v0.3.0" / "schema.json"


@dataclass
class SyncPreview:
    """Preview of what will be synced"""

    regulation_id: str
    annotations_to_sync: list[dict]
    conversions: list[ConversionResult]
    target_file: Path | None
    errors: list[str]


@dataclass
class SyncResult:
    """Result of sync execution"""

    regulation_id: str
    synced_count: int
    errors: list[str]
    target_file: Path | None


def load_annotations(reg_id: str) -> list[dict]:
    """Load annotations from YAML file."""
    annotations_file = ANNOTATIONS_DIR / f"{reg_id}.yaml"
    if not annotations_file.exists():
        return []

    with open(annotations_file, encoding="utf-8") as f:
        data = yaml.safe_load(f) or {}

    return data.get("annotations", [])


def save_annotations(reg_id: str, annotations: list[dict]) -> None:
    """Save annotations back to YAML file."""
    annotations_file = ANNOTATIONS_DIR / f"{reg_id}.yaml"
    ANNOTATIONS_DIR.mkdir(exist_ok=True)

    with open(annotations_file, "w", encoding="utf-8") as f:
        yaml.dump(
            {"annotations": annotations},
            f,
            allow_unicode=True,
            sort_keys=False,
            default_flow_style=False,
        )


def find_regulation_file(reg_id: str) -> Path | None:
    """Find the regulation YAML file for a given ID."""
    # Search in all subdirectories
    for yaml_file in REGULATION_DIR.rglob("*.yaml"):
        try:
            with open(yaml_file, encoding="utf-8") as f:
                data = yaml.safe_load(f)
                if data and data.get("$id") == reg_id:
                    return yaml_file
        except Exception:
            continue
    return None


def load_regulation(reg_file: Path) -> dict:
    """Load a regulation YAML file."""
    with open(reg_file, encoding="utf-8") as f:
        return yaml.safe_load(f) or {}


def save_regulation(reg_file: Path, data: dict) -> None:
    """Save a regulation YAML file."""
    with open(reg_file, "w", encoding="utf-8") as f:
        yaml.dump(
            data, f, allow_unicode=True, sort_keys=False, default_flow_style=False
        )


def validate_against_schema(data: dict) -> list[str]:
    """
    Validate regulation data against JSON schema.

    Returns list of validation errors (empty if valid).
    """
    if not SCHEMA_PATH.exists():
        return ["Schema file not found"]

    try:
        import jsonschema

        with open(SCHEMA_PATH, encoding="utf-8") as f:
            schema = json.load(f)

        validator = jsonschema.Draft7Validator(schema)
        errors = list(validator.iter_errors(data))
        return [f"{e.path}: {e.message}" for e in errors[:5]]  # Limit to 5 errors
    except ImportError:
        # jsonschema not installed, skip validation
        return []
    except Exception as e:
        return [f"Validation error: {e}"]


def get_approved_annotations(annotations: list[dict]) -> list[dict]:
    """Filter annotations to only those with status: approved."""
    return [a for a in annotations if a.get("status") == "approved"]


def group_by_article(annotations: list[dict]) -> dict[str, list[dict]]:
    """Group annotations by article number."""
    by_article: dict[str, list[dict]] = {}
    for ann in annotations:
        article_nr = ann.get("target", {}).get("article", "")
        if article_nr:
            if article_nr not in by_article:
                by_article[article_nr] = []
            by_article[article_nr].append(ann)
    return by_article


def preview_sync(reg_id: str) -> SyncPreview:
    """
    Preview what will be synced for a regulation.

    Returns SyncPreview with all annotations that would be synced.
    """
    errors: list[str] = []
    conversions: list[ConversionResult] = []

    # Load annotations
    annotations = load_annotations(reg_id)
    if not annotations:
        return SyncPreview(
            regulation_id=reg_id,
            annotations_to_sync=[],
            conversions=[],
            target_file=None,
            errors=["No annotations file found"],
        )

    # Filter approved
    approved = get_approved_annotations(annotations)
    if not approved:
        return SyncPreview(
            regulation_id=reg_id,
            annotations_to_sync=[],
            conversions=[],
            target_file=None,
            errors=["No approved annotations to sync"],
        )

    # Find target file
    target_file = find_regulation_file(reg_id)
    if not target_file:
        errors.append(f"Regulation file not found for: {reg_id}")

    # Convert annotations
    converter = AnnotationConverter()
    for ann in approved:
        result = converter.convert(ann)
        conversions.append(result)
        if not result.success:
            errors.append(f"Conversion error: {result.error}")

    return SyncPreview(
        regulation_id=reg_id,
        annotations_to_sync=approved,
        conversions=conversions,
        target_file=target_file,
        errors=errors,
    )


def execute_sync(reg_id: str) -> SyncResult:
    """
    Execute sync for a regulation.

    Applies approved annotations to the law YAML and marks them as promoted.
    """
    errors: list[str] = []
    synced_count = 0

    # Load annotations
    all_annotations = load_annotations(reg_id)
    if not all_annotations:
        return SyncResult(
            regulation_id=reg_id,
            synced_count=0,
            errors=["No annotations file found"],
            target_file=None,
        )

    # Filter approved
    approved = get_approved_annotations(all_annotations)
    if not approved:
        return SyncResult(
            regulation_id=reg_id,
            synced_count=0,
            errors=["No approved annotations to sync"],
            target_file=None,
        )

    # Find and load regulation file
    reg_file = find_regulation_file(reg_id)
    if not reg_file:
        return SyncResult(
            regulation_id=reg_id,
            synced_count=0,
            errors=[f"Regulation file not found for: {reg_id}"],
            target_file=None,
        )

    reg_data = load_regulation(reg_file)
    articles = reg_data.get("articles", [])

    # Group annotations by article
    by_article = group_by_article(approved)
    converter = AnnotationConverter()

    # Apply conversions
    for article_nr, article_annotations in by_article.items():
        # Find article in regulation
        article = None
        for art in articles:
            if str(art.get("number")) == str(article_nr):
                article = art
                break

        if not article:
            errors.append(f"Article {article_nr} not found in regulation")
            continue

        for ann in article_annotations:
            result = converter.convert(ann)
            if result.success:
                apply_conversion_to_article(article, result)
                # Mark as promoted in annotations list
                for orig_ann in all_annotations:
                    if orig_ann.get("target", {}).get("selector", {}).get(
                        "exact"
                    ) == ann.get("target", {}).get("selector", {}).get(
                        "exact"
                    ) and orig_ann.get("target", {}).get("article") == ann.get(
                        "target", {}
                    ).get("article"):
                        mark_annotation_promoted(orig_ann)
                        synced_count += 1
                        break
            else:
                errors.append(f"Conversion failed: {result.error}")

    # Validate result
    validation_errors = validate_against_schema(reg_data)
    if validation_errors:
        errors.extend(validation_errors)
        # Don't save if validation fails
        return SyncResult(
            regulation_id=reg_id,
            synced_count=0,
            errors=errors,
            target_file=reg_file,
        )

    # Save updated regulation
    save_regulation(reg_file, reg_data)

    # Save updated annotations (with promoted status)
    save_annotations(reg_id, all_annotations)

    return SyncResult(
        regulation_id=reg_id,
        synced_count=synced_count,
        errors=errors,
        target_file=reg_file,
    )


def list_all_regulations() -> list[str]:
    """List all regulation IDs that have annotation files."""
    reg_ids = []
    if ANNOTATIONS_DIR.exists():
        for yaml_file in ANNOTATIONS_DIR.glob("*.yaml"):
            reg_ids.append(yaml_file.stem)
    return reg_ids


def main():
    parser = argparse.ArgumentParser(description="Sync W3C annotations to law YAMLs")
    parser.add_argument(
        "regulation_id",
        nargs="?",
        help="Regulation ID to sync (e.g., participatiewet)",
    )
    parser.add_argument(
        "--all",
        action="store_true",
        help="Sync all regulations with annotation files",
    )
    parser.add_argument(
        "--preview",
        action="store_true",
        help="Preview what would be synced (dry run)",
    )
    parser.add_argument(
        "--execute",
        action="store_true",
        help="Execute the sync",
    )

    args = parser.parse_args()

    if not args.regulation_id and not args.all:
        parser.print_help()
        sys.exit(1)

    if args.all:
        reg_ids = list_all_regulations()
        if not reg_ids:
            print("No annotation files found")
            sys.exit(0)
    else:
        reg_ids = [args.regulation_id]

    for reg_id in reg_ids:
        print(f"\n{'=' * 60}")
        print(f"Regulation: {reg_id}")
        print("=" * 60)

        if args.preview or (not args.preview and not args.execute):
            preview = preview_sync(reg_id)
            print(f"\nAnnotations to sync: {len(preview.annotations_to_sync)}")

            for i, (ann, conv) in enumerate(
                zip(preview.annotations_to_sync, preview.conversions)
            ):
                exact = ann.get("target", {}).get("selector", {}).get("exact", "")[:40]
                article = ann.get("target", {}).get("article", "?")
                status = "OK" if conv.success else "FAIL"
                print(f'  [{status}] Art. {article}: "{exact}..." → {conv.target_path}')

            if preview.errors:
                print("\nErrors:")
                for err in preview.errors:
                    print(f"  - {err}")

            if preview.target_file:
                print(f"\nTarget file: {preview.target_file}")

        if args.execute:
            result = execute_sync(reg_id)
            print(f"\nSync result: {result.synced_count} annotations synced")

            if result.errors:
                print("\nErrors:")
                for err in result.errors:
                    print(f"  - {err}")

            if result.target_file:
                print(f"\nUpdated: {result.target_file}")


if __name__ == "__main__":
    main()

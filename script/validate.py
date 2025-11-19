#!/usr/bin/env python3
"""
Validate regelrecht YAML law files against the JSON schema.

Usage:
    uv run python script/validate.py regulation/nl/wet/wet_op_de_zorgtoeslag/2025-01-01.yaml
    uv run python script/validate.py regulation/nl/**/*.yaml
"""

import sys
import json
from pathlib import Path
import yaml
from jsonschema import validate, ValidationError
import requests

# Configure UTF-8 encoding for emoji support on Windows
if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8")  # type: ignore[attr-defined]


def load_schema(schema_path_or_url: str) -> dict:
    """Load JSON schema from file or URL."""
    if schema_path_or_url.startswith("http://") or schema_path_or_url.startswith(
        "https://"
    ):
        response = requests.get(schema_path_or_url)
        response.raise_for_status()
        return response.json()
    else:
        with open(schema_path_or_url, "r") as f:
            return json.load(f)


def validate_law_file(yaml_path: Path, schema: dict) -> tuple[bool, list[str]]:
    """
    Validate a YAML law file against the schema.

    Returns:
        (is_valid, errors) tuple
    """
    errors = []

    try:
        # Load YAML file
        with open(yaml_path, "r", encoding="utf-8") as f:
            law_data = yaml.safe_load(f)

        # Validate against schema
        validate(instance=law_data, schema=schema)

        return (True, [])

    except yaml.YAMLError as e:
        errors.append(f"YAML parsing error: {e}")
        return (False, errors)

    except ValidationError as e:
        errors.append(f"Schema validation error: {e.message}")
        if e.path:
            path_str = " -> ".join(str(p) for p in e.path)
            errors.append(f"  At: {path_str}")
        return (False, errors)

    except Exception as e:
        errors.append(f"Unexpected error: {e}")
        return (False, errors)


def main():
    if len(sys.argv) < 2:
        print("Usage: uv run python script/validate.py <yaml_file_or_pattern>")
        print()
        print("Examples:")
        print(
            "  uv run python script/validate.py regulation/nl/wet/wet_op_de_zorgtoeslag/2025-01-01.yaml"
        )
        print('  uv run python script/validate.py "regulation/nl/**/*.yaml"')
        sys.exit(1)

    # Load schema
    schema_url = "https://raw.githubusercontent.com/MinBZK/poc-machine-law/refs/heads/main/schema/v0.2.0/schema.json"

    # Check if schema exists locally first
    local_schema = Path("schema/v0.2.0/schema.json")
    if local_schema.exists():
        print(f"📋 Loading schema from: {local_schema}")
        schema = load_schema(str(local_schema))
    else:
        print(f"📋 Loading schema from: {schema_url}")
        try:
            schema = load_schema(schema_url)
        except Exception as e:
            print(f"❌ Failed to load schema: {e}")
            sys.exit(1)

    # Find YAML files to validate
    pattern = sys.argv[1]
    yaml_files = []

    if "*" in pattern:
        # Glob pattern
        parts = pattern.split("/")
        base_dir = Path(
            "/".join(parts[: parts.index("**")])
            if "**" in parts
            else "/".join(parts[:-1])
        )
        glob_pattern = (
            "/".join(parts[parts.index("**") :]) if "**" in parts else parts[-1]
        )
        yaml_files = list(base_dir.glob(glob_pattern))
    else:
        # Single file
        yaml_files = [Path(pattern)]

    if not yaml_files:
        print(f"❌ No YAML files found matching: {pattern}")
        sys.exit(1)

    print(f"🔍 Validating {len(yaml_files)} file(s)...")
    print()

    # Validate each file
    total = len(yaml_files)
    valid_count = 0
    invalid_count = 0

    for yaml_file in sorted(yaml_files):
        if not yaml_file.exists():
            print(f"❌ {yaml_file}: File not found")
            invalid_count += 1
            continue

        is_valid, errors = validate_law_file(yaml_file, schema)

        if is_valid:
            print(f"✅ {yaml_file}: Valid")
            valid_count += 1
        else:
            print(f"❌ {yaml_file}: Invalid")
            for error in errors:
                print(f"   {error}")
            invalid_count += 1

    # Summary
    print()
    print("=" * 60)
    print("📊 Validation Summary:")
    print(f"   Total files: {total}")
    print(f"   ✅ Valid: {valid_count}")
    print(f"   ❌ Invalid: {invalid_count}")
    print("=" * 60)

    if invalid_count > 0:
        sys.exit(1)
    else:
        print()
        print("🎉 All files are valid!")
        sys.exit(0)


if __name__ == "__main__":
    main()

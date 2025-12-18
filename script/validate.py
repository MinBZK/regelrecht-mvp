#!/usr/bin/env python3
"""
Validate regelrecht YAML law files against the JSON schema.

The schema version is read from the $schema property in each YAML file,
allowing files to declare which schema version they conform to.

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

# Cache for loaded schemas
_schema_cache: dict[str, dict] = {}


def load_schema(schema_path_or_url: str) -> dict:
    """Load JSON schema from file or URL, with caching."""
    if schema_path_or_url in _schema_cache:
        return _schema_cache[schema_path_or_url]

    if schema_path_or_url.startswith("http://") or schema_path_or_url.startswith(
        "https://"
    ):
        response = requests.get(schema_path_or_url)
        response.raise_for_status()
        schema = response.json()
    else:
        with open(schema_path_or_url, "r") as f:
            schema = json.load(f)

    _schema_cache[schema_path_or_url] = schema
    return schema


def get_local_schema_path(schema_url: str) -> Path | None:
    """
    Try to find a local schema file based on the URL.

    Example: https://.../schema/v0.3.0/schema.json -> schema/v0.3.0/schema.json
    """
    if "/schema/" in schema_url:
        # Extract the schema path from the URL
        parts = schema_url.split("/schema/")
        if len(parts) == 2:
            local_path = Path("schema") / parts[1]
            if local_path.exists():
                return local_path
    return None


def validate_law_file(yaml_path: Path) -> tuple[bool, list[str]]:
    """
    Validate a YAML law file against the schema declared in its $schema property.

    Returns:
        (is_valid, errors) tuple
    """
    errors = []

    try:
        # Load YAML file
        with open(yaml_path, "r", encoding="utf-8") as f:
            law_data = yaml.safe_load(f)

        # Get schema URL from the file
        schema_url = law_data.get("$schema")
        if not schema_url:
            errors.append("No $schema property found in YAML file")
            return (False, errors)

        # Try to load schema locally first, fall back to URL
        local_path = get_local_schema_path(schema_url)
        if local_path:
            schema = load_schema(str(local_path))
        else:
            schema = load_schema(schema_url)

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

    except requests.RequestException as e:
        errors.append(f"Failed to load schema: {e}")
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
        print()
        print(
            "Note: Schema version is read from the $schema property in each YAML file."
        )
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

        is_valid, errors = validate_law_file(yaml_file)

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

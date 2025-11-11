#!/usr/bin/env python3
"""
Fix YAML law files to match v0.2.0 schema structure.

Moves execution-related fields under machine_readable.execution
"""

import sys
from pathlib import Path
import yaml


def fix_source_format(source):
    """
    Fix source format to match schema.

    Schema allows: article, regeling, field
    Old format uses: ref, parameters

    Convert:
      ref: regulation/nl/wet/awir#toetsingsinkomen
      parameters: {BSN: $BSN}
    To:
      article: awir.toetsingsinkomen

    Note: Schema doesn't support passing parameters, so we drop them.
    """
    if not isinstance(source, dict):
        return source

    if "ref" in source:
        ref = source["ref"]

        # Handle internal references (start with #)
        if ref.startswith("#"):
            # Internal reference like '#vermogen_onder_grens'
            # Keep as-is but rename 'ref' to 'article'
            return {"article": ref}

        # Parse external references
        # Format: regulation/nl/wet/law_name#endpoint
        # or: regulation/nl/ministeriele_regeling/law_name#endpoint
        if "#" in ref:
            path, field = ref.split("#", 1)
            parts = path.split("/")

            # Extract law type and name
            if len(parts) >= 4:
                law_type = parts[2]  # 'wet' or 'ministeriele_regeling'
                law_name = parts[3] if len(parts) > 3 else ""

                # Convert to article format: law_name.field
                return {"article": f"{law_name}.{field}"}

        # Fallback: just remove 'ref' prefix and parameters
        return {"article": ref.replace("regulation/nl/wet/", "").replace("regulation/nl/ministeriele_regeling/", "")}

    # If no 'ref', return as-is (might already be in correct format)
    return source


def fix_machine_readable_structure(article):
    """
    Fix machine_readable structure to match schema.

    Schema requires:
    machine_readable:
      execution:
        parameters: [...]
        input: [...]
        output: [...]
        actions: [...]

    Current files have these fields directly under machine_readable.
    """
    if "machine_readable" not in article:
        return article

    mr = article["machine_readable"]

    # Fields that should be under execution
    execution_fields = ["parameters", "input", "output", "actions", "produces"]

    # Check if any execution fields are at wrong level
    needs_fix = any(field in mr for field in execution_fields)

    if needs_fix:
        # Create execution section if it doesn't exist
        if "execution" not in mr:
            mr["execution"] = {}

        # Move fields into execution
        for field in execution_fields:
            if field in mr:
                mr["execution"][field] = mr.pop(field)

    # Fix source formats in input fields
    if "execution" in mr and "input" in mr["execution"]:
        for input_field in mr["execution"]["input"]:
            if "source" in input_field:
                input_field["source"] = fix_source_format(input_field["source"])

    return article


def fix_law_file(yaml_path):
    """Fix a single YAML law file."""
    print(f"🔧 Fixing {yaml_path}")

    with open(yaml_path, "r", encoding="utf-8") as f:
        law_data = yaml.safe_load(f)

    # Fix each article
    changes = 0
    if "articles" in law_data:
        for article in law_data["articles"]:
            if "machine_readable" in article:
                before = str(article["machine_readable"])
                article = fix_machine_readable_structure(article)
                after = str(article["machine_readable"])
                if before != after:
                    changes += 1

    if changes > 0:
        # Save fixed file
        with open(yaml_path, "w", encoding="utf-8") as f:
            yaml.dump(
                law_data,
                f,
                allow_unicode=True,
                sort_keys=False,
                default_flow_style=False,
                width=100,
            )
        print(f"   ✅ Fixed {changes} article(s)")
    else:
        print(f"   ℹ️  No changes needed")

    return changes


def main():
    if len(sys.argv) < 2:
        print("Usage: uv run python script/fix_schema.py <yaml_file> [yaml_file...]")
        print()
        print("Example:")
        print('  uv run python script/fix_schema.py regulation/nl/wet/wet_op_de_zorgtoeslag/2025-01-01.yaml')
        print('  uv run python script/fix_schema.py regulation/nl/**/*.yaml')
        sys.exit(1)

    files = []
    for pattern in sys.argv[1:]:
        if "*" in pattern:
            # Glob pattern
            parts = pattern.split("/")
            if "**" in parts:
                base_idx = parts.index("**")
                base_dir = Path("/".join(parts[:base_idx]))
                glob_pattern = "/".join(parts[base_idx:])
                files.extend(base_dir.glob(glob_pattern))
            else:
                base_dir = Path("/".join(parts[:-1]))
                glob_pattern = parts[-1]
                files.extend(base_dir.glob(glob_pattern))
        else:
            files.append(Path(pattern))

    if not files:
        print(f"❌ No files found")
        sys.exit(1)

    print(f"📁 Processing {len(files)} file(s)...\n")

    total_changes = 0
    for yaml_path in files:
        if not yaml_path.exists():
            print(f"⚠️  Skipping {yaml_path}: not found")
            continue

        changes = fix_law_file(yaml_path)
        total_changes += changes

    print()
    print("=" * 60)
    print(f"✅ Complete! Fixed {total_changes} article(s) across {len(files)} file(s)")
    print("=" * 60)
    print()
    print("Next step: Validate all files")
    print('  uv run python script/validate.py "regulation/nl/**/*.yaml"')


if __name__ == "__main__":
    main()

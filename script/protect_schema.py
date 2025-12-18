#!/usr/bin/env python3
"""Pre-commit hook to protect existing schema version files from modification.

Schema versions (schema/v*/) are immutable once released. This hook blocks
modifications or deletions of schema.json files in versioned directories.

The schema/latest/ directory is NOT protected and can be updated freely.
"""

import re
import subprocess
import sys

VERSION_PATTERN = re.compile(r"^schema/v[0-9]+\.[0-9]+\.[0-9]+/")


def main() -> int:
    """Check for modifications to existing versioned schema files."""
    result = subprocess.run(
        ["git", "diff", "--cached", "--name-status", "--", "schema/"],
        capture_output=True,
        text=True,
    )

    violations: list[str] = []
    for line in result.stdout.strip().split("\n"):
        if not line:
            continue
        status, filepath = line.split("\t", 1)
        # Only protect versioned directories (schema/vX.X.X/)
        if status in ("M", "D") and VERSION_PATTERN.match(filepath):
            violations.append(f"{status}\t{filepath}")

    if violations:
        print("Schema Protection Error!")
        print("=" * 50)
        print()
        print("Cannot modify or delete files in versioned schema directories:")
        for v in violations:
            print(f"  {v}")
        print()
        print("Schema versions are immutable. Create a new version instead.")
        print("(schema/latest/ can be updated freely)")
        return 1

    return 0


if __name__ == "__main__":
    sys.exit(main())

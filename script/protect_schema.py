#!/usr/bin/env python3
"""Pre-commit hook to protect existing schema files from modification.

Schema versions are immutable once released. This hook blocks modifications
or deletions of existing schema.json files. Adding new schema versions is allowed.
"""

import subprocess
import sys


def main() -> int:
    """Check for modifications to existing schema.json files."""
    result = subprocess.run(
        ["git", "diff", "--cached", "--name-status", "--", "schema/**/schema.json"],
        capture_output=True,
        text=True,
    )

    violations: list[str] = []
    for line in result.stdout.strip().split("\n"):
        if not line:
            continue
        status, filepath = line.split("\t", 1)
        if status in ("M", "D"):  # Modified or Deleted
            violations.append(f"{status}\t{filepath}")

    if violations:
        print("Schema Protection Error!")
        print("=" * 50)
        print()
        print("Cannot modify or delete existing schema files:")
        for v in violations:
            print(f"  {v}")
        print()
        print("Schema versions are immutable. Create a new version instead.")
        return 1

    return 0


if __name__ == "__main__":
    sys.exit(main())

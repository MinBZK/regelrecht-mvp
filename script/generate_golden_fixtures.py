#!/usr/bin/env python3
"""
Golden Fixture Generator

Generates JSON test fixtures by executing test cases against the Python engine.
These fixtures are used by the Rust engine to verify identical behavior.
"""

import json
import shutil
import sys
import tempfile
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

import yaml

# Add parent directory to path for imports
sys.path.insert(0, str(Path(__file__).parent.parent))

from script.golden_test_definitions import ALL_TEST_CATEGORIES

from engine.article_loader import ArticleBasedLaw
from engine.engine import ArticleEngine
from engine.service import LawExecutionService


class MockServiceProvider:
    """Mock service provider for single-law tests."""

    def evaluate_uri(self, uri: str, parameters: dict, calculation_date: str) -> Any:
        raise ValueError(f"Cannot resolve URI {uri} in mock provider")


def normalize_value(value: Any) -> Any:
    """
    Normalize a value for JSON serialization and deterministic comparison.

    - Floats are rounded to 6 decimal places
    - None becomes null in JSON
    - Dicts and lists are recursively normalized
    """
    if value is None:
        return None
    elif isinstance(value, bool):
        return value
    elif isinstance(value, float):
        # Round to 6 decimal places for deterministic comparison
        return round(value, 6)
    elif isinstance(value, int):
        return value
    elif isinstance(value, str):
        return value
    elif isinstance(value, dict):
        return {k: normalize_value(v) for k, v in value.items()}
    elif isinstance(value, list):
        return [normalize_value(v) for v in value]
    else:
        # Convert to string for unknown types
        return str(value)


def execute_single_law_test(test_def: dict) -> dict:
    """Execute a test that only requires a single law."""
    law_yaml = test_def["law_yaml"]
    output_name = test_def["output_name"]
    parameters = test_def.get("parameters", {})
    calculation_date = test_def.get("calculation_date", "2025-01-01")
    expect_error = test_def.get("expect_error", False)

    # Parse the law YAML
    law_data = yaml.safe_load(law_yaml)
    law = ArticleBasedLaw(law_data)

    # Find the article with the output
    article = None
    for art in law.articles:
        if output_name in art.get_output_names():
            article = art
            break

    if article is None:
        return {
            "success": False,
            "error_type": "OutputNotFound",
            "error_message": f"No article found with output '{output_name}'",
        }

    # Create engine and execute
    engine = ArticleEngine(article, law)
    mock_service = MockServiceProvider()

    try:
        # Don't filter by output_name - calculate all outputs to ensure dependencies work
        result = engine.evaluate(parameters, mock_service, calculation_date)
        return {
            "success": True,
            "article_number": result.article_number,
            "outputs": normalize_value(result.output),
            "resolved_inputs": normalize_value(result.input),
        }
    except ZeroDivisionError:
        if expect_error:
            return {
                "success": False,
                "error_type": "DivisionByZero",
                "error_message": "Division by zero",
            }
        raise
    except Exception as e:
        if expect_error:
            error_type = type(e).__name__
            return {
                "success": False,
                "error_type": error_type,
                "error_message": str(e),
            }
        raise


def execute_multi_law_test(test_def: dict) -> dict:
    """Execute a test that requires multiple laws."""
    laws = test_def["laws"]
    law_id = test_def["law_id"]
    output_name = test_def["output_name"]
    parameters = test_def.get("parameters", {})
    calculation_date = test_def.get("calculation_date", "2025-01-01")
    expect_error = test_def.get("expect_error", False)

    # Create temporary directory with law files
    with tempfile.TemporaryDirectory() as tmpdir:
        # Write each law to the temp directory
        for law_spec in laws:
            law_yaml = law_spec["yaml"]
            law_data = yaml.safe_load(law_yaml)
            law_id_local = law_data.get("$id", law_spec.get("law_id", "unknown"))

            # Create law directory structure
            law_dir = Path(tmpdir) / "wet" / law_id_local
            law_dir.mkdir(parents=True, exist_ok=True)

            # Write law file
            law_file = law_dir / "2025-01-01.yaml"
            law_file.write_text(law_yaml)

        # Create service with temp directory
        service = LawExecutionService(tmpdir)

        try:
            result = service.evaluate_law_output(
                law_id=law_id,
                output_name=output_name,
                parameters=parameters,
                calculation_date=calculation_date,
            )
            return {
                "success": True,
                "article_number": result.article_number,
                "outputs": normalize_value(result.output),
                "resolved_inputs": normalize_value(result.input),
            }
        except ZeroDivisionError:
            if expect_error:
                return {
                    "success": False,
                    "error_type": "DivisionByZero",
                    "error_message": "Division by zero",
                }
            raise
        except ValueError as e:
            if expect_error:
                # Map error to type
                error_msg = str(e)
                if "Could not resolve URI" in error_msg:
                    error_type = "LawNotFound"
                else:
                    error_type = "ValueError"
                return {
                    "success": False,
                    "error_type": error_type,
                    "error_message": error_msg,
                }
            raise
        except Exception as e:
            if expect_error:
                error_type = type(e).__name__
                return {
                    "success": False,
                    "error_type": error_type,
                    "error_message": str(e),
                }
            raise


def execute_test(test_def: dict) -> dict:
    """Execute a test definition and capture the result."""
    if test_def.get("multi_law", False):
        return execute_multi_law_test(test_def)
    else:
        return execute_single_law_test(test_def)


def generate_fixture_file(category: str, tests: list[dict], output_dir: Path) -> Path:
    """Generate a fixture file for a category of tests."""
    test_cases = []

    for test_def in tests:
        print(f"  Executing: {test_def['id']}...")

        try:
            expected = execute_test(test_def)

            test_case = {
                "id": test_def["id"],
                "description": test_def["description"],
                "category": category,
                "parameters": normalize_value(test_def.get("parameters", {})),
                "calculation_date": test_def.get("calculation_date", "2025-01-01"),
                "expected": expected,
            }

            # Add law YAML (single law) or laws array (multi-law)
            if test_def.get("multi_law", False):
                test_case["multi_law"] = True
                test_case["laws"] = test_def["laws"]
                test_case["law_id"] = test_def["law_id"]
                test_case["output_name"] = test_def["output_name"]
            else:
                test_case["law_yaml"] = test_def["law_yaml"]
                test_case["law_id"] = test_def["law_id"]
                test_case["output_name"] = test_def["output_name"]

            test_cases.append(test_case)
            print(
                f"    OK: {expected.get('outputs', expected.get('error_type', 'N/A'))}"
            )

        except Exception as e:
            print(f"    ERROR: {e}")
            # Still add the test case but mark as generator error
            test_case = {
                "id": test_def["id"],
                "description": test_def["description"],
                "category": category,
                "generator_error": str(e),
            }
            test_cases.append(test_case)

    # Build fixture file content
    fixture = {
        "version": "1.0.0",
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "category": category,
        "test_count": len(test_cases),
        "test_cases": test_cases,
    }

    # Write fixture file
    output_file = output_dir / f"{category}.json"
    with open(output_file, "w", encoding="utf-8") as f:
        json.dump(fixture, f, indent=2, ensure_ascii=False)

    return output_file


def copy_to_rust_fixtures(source_dir: Path, project_root: Path) -> Path | None:
    """Copy fixtures to Rust engine tests directory if it exists.

    Returns the Rust fixtures path if copied, None otherwise.
    """
    # Check for Rust engine in worktree or direct location
    rust_locations = [
        project_root
        / ".worktrees"
        / "rust-engine"
        / "packages"
        / "engine"
        / "tests"
        / "fixtures",
        project_root / "packages" / "engine" / "tests" / "fixtures",
    ]

    for rust_fixtures_dir in rust_locations:
        rust_engine_dir = rust_fixtures_dir.parent.parent
        if rust_engine_dir.exists():
            rust_fixtures_dir.mkdir(parents=True, exist_ok=True)
            # Copy all JSON files
            for json_file in source_dir.glob("*.json"):
                shutil.copy2(json_file, rust_fixtures_dir / json_file.name)
            return rust_fixtures_dir

    return None


def main() -> int:
    """Generate all golden test fixtures."""
    print("Golden Test Fixture Generator")
    print("=" * 60)

    project_root = Path(__file__).parent.parent

    # Create output directory for Python tests
    output_dir = project_root / "tests" / "golden_fixtures"
    output_dir.mkdir(parents=True, exist_ok=True)

    print(f"Output directory: {output_dir}")
    print()

    # Generate fixtures for each category
    total_tests = 0
    total_errors = 0

    for category, tests in ALL_TEST_CATEGORIES.items():
        print(f"Category: {category} ({len(tests)} tests)")
        print("-" * 40)

        fixture_file = generate_fixture_file(category, tests, output_dir)

        # Count successes and errors
        with open(fixture_file) as f:
            fixture_data = json.load(f)
            for tc in fixture_data["test_cases"]:
                total_tests += 1
                if "generator_error" in tc:
                    total_errors += 1

        print(f"  Written: {fixture_file}")
        print()

    # Generate combined fixture file (all tests in one file)
    # NOTE: This combined file is optional - the Rust tests use per-category files.
    # It's useful for debugging and tools that want to process all tests at once.
    print("Generating combined fixture file...")
    all_test_cases = []
    for category, tests in ALL_TEST_CATEGORIES.items():
        fixture_file = output_dir / f"{category}.json"
        with open(fixture_file) as f:
            fixture_data = json.load(f)
            all_test_cases.extend(fixture_data["test_cases"])

    combined_fixture = {
        "version": "1.0.0",
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "category": "all",
        "test_count": len(all_test_cases),
        "test_cases": all_test_cases,
    }

    combined_file = output_dir / "all_tests.json"
    with open(combined_file, "w", encoding="utf-8") as f:
        json.dump(combined_fixture, f, indent=2, ensure_ascii=False)
    print(f"  Written: {combined_file}")

    # Copy to Rust fixtures directory if it exists
    rust_dir = copy_to_rust_fixtures(output_dir, project_root)

    # Print summary
    print()
    print("=" * 60)
    print("Summary")
    print("=" * 60)
    print(f"  Total tests: {total_tests}")
    print(f"  Successful: {total_tests - total_errors}")
    print(f"  Errors: {total_errors}")
    print()
    print(f"Fixture files written to: {output_dir}")
    if rust_dir:
        print(f"Also copied to Rust: {rust_dir}")
    else:
        print("Note: Rust engine not found, fixtures not copied for Rust tests")

    return 0 if total_errors == 0 else 1


if __name__ == "__main__":
    sys.exit(main())

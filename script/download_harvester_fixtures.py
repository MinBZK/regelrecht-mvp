"""Download XML fixtures for harvester integration tests."""

import argparse
import sys
from pathlib import Path

# Add project root to path so we can import harvester
sys.path.insert(0, str(Path(__file__).parent.parent))

import requests

FIXTURES_DIR = Path(__file__).parent.parent / "tests" / "test_harvester" / "fixtures"
BWB_REPOSITORY_URL = "https://repository.officiele-overheidspublicaties.nl/bwb"

# Available fixtures for download
FIXTURES = [
    ("BWBR0018451", "2025-01-01", "zorgtoeslag"),
    ("BWBR0004627", "2025-08-01", "kieswet"),
    ("BWBR0035917", "2025-07-05", "wlz"),
    ("BWBR0018450", "2025-07-05", "zvw"),
    ("BWBR0018472", "2025-01-01", "awir"),
]


def download_fixture(bwb_id: str, date: str, name: str) -> None:
    """Download WTI and Toestand XML for a law.

    Args:
        bwb_id: BWB identifier (e.g., "BWBR0018451")
        date: Effective date in YYYY-MM-DD format
        name: Short name for the fixture files (e.g., "zorgtoeslag")
    """
    # Download Toestand (with _0 suffix for consolidated version)
    toestand_url = f"{BWB_REPOSITORY_URL}/{bwb_id}/{date}_0/xml/{bwb_id}_{date}_0.xml"
    print(f"Downloading Toestand from {toestand_url}")
    r = requests.get(toestand_url, timeout=30)
    r.raise_for_status()

    toestand_path = FIXTURES_DIR / f"{name}_toestand.xml"
    toestand_path.write_bytes(r.content)
    print(f"  Saved: {toestand_path} ({len(r.content)} bytes)")

    # Download WTI
    wti_url = f"{BWB_REPOSITORY_URL}/{bwb_id}/{bwb_id}.WTI"
    print(f"Downloading WTI from {wti_url}")
    r = requests.get(wti_url, timeout=30)
    r.raise_for_status()

    wti_path = FIXTURES_DIR / f"{name}_wti.xml"
    wti_path.write_bytes(r.content)
    print(f"  Saved: {wti_path} ({len(r.content)} bytes)")


def main() -> None:
    """Download fixtures for harvester tests."""
    parser = argparse.ArgumentParser(
        description="Download XML fixtures for harvester integration tests."
    )
    parser.add_argument(
        "--date",
        "-d",
        default="2025-01-01",
        help="Effective date in YYYY-MM-DD format (default: 2025-01-01)",
    )
    parser.add_argument(
        "--law",
        "-l",
        choices=[name for _, _, name in FIXTURES],
        help="Download only a specific law fixture (default: all)",
    )
    args = parser.parse_args()

    fixtures_to_download = FIXTURES
    if args.law:
        fixtures_to_download = [
            (bwb, date, name) for bwb, date, name in FIXTURES if name == args.law
        ]

    for bwb_id, default_date, name in fixtures_to_download:
        date = args.date if args.date != "2025-01-01" else default_date
        download_fixture(bwb_id, date, name)

    print("\nDone! Run 'just update-harvester-fixtures' to regenerate expected YAML.")


if __name__ == "__main__":
    main()

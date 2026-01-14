"""Shared configuration for the harvester module."""

import re
from datetime import datetime

# Base URL for BWB (Basiswettenbestand) repository
BWB_REPOSITORY_URL = "https://repository.officiele-overheidspublicaties.nl/bwb"

# HTTP timeout in seconds (10s is reasonable for government APIs)
HTTP_TIMEOUT = 10

# BWB ID pattern: BWBR followed by 7 digits
BWB_ID_PATTERN = re.compile(r"^BWBR\d{7}$")


def validate_bwb_id(bwb_id: str) -> None:
    """Validate BWB ID format.

    Args:
        bwb_id: The BWB identifier to validate

    Raises:
        ValueError: If BWB ID format is invalid
    """
    if not BWB_ID_PATTERN.match(bwb_id):
        raise ValueError(
            f"Invalid BWB ID format: '{bwb_id}'. Expected BWBRXXXXXXX (e.g., BWBR0018451)"
        )


def validate_date(date_str: str) -> None:
    """Validate date format.

    Args:
        date_str: Date string to validate

    Raises:
        ValueError: If date format is invalid
    """
    try:
        datetime.strptime(date_str, "%Y-%m-%d")
    except ValueError as e:
        raise ValueError(
            f"Invalid date format: '{date_str}'. Expected YYYY-MM-DD (e.g., 2025-01-01)"
        ) from e

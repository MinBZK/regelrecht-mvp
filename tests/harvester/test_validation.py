"""Tests for input validation."""

import pytest

from harvester.config import validate_bwb_id, validate_date


class TestValidateBwbId:
    """Tests for BWB ID validation."""

    def test_valid_bwb_id(self) -> None:
        validate_bwb_id("BWBR0018451")  # Should not raise

    def test_invalid_prefix(self) -> None:
        with pytest.raises(ValueError, match="Invalid BWB ID format"):
            validate_bwb_id("BWBA0018451")

    def test_too_few_digits(self) -> None:
        with pytest.raises(ValueError, match="Invalid BWB ID format"):
            validate_bwb_id("BWBR001845")

    def test_too_many_digits(self) -> None:
        with pytest.raises(ValueError, match="Invalid BWB ID format"):
            validate_bwb_id("BWBR00184511")

    def test_lowercase_rejected(self) -> None:
        with pytest.raises(ValueError, match="Invalid BWB ID format"):
            validate_bwb_id("bwbr0018451")

    def test_empty_string(self) -> None:
        with pytest.raises(ValueError, match="Invalid BWB ID format"):
            validate_bwb_id("")


class TestValidateDate:
    """Tests for date validation."""

    def test_valid_date(self) -> None:
        validate_date("2025-01-01")  # Should not raise

    def test_invalid_format_dmy(self) -> None:
        with pytest.raises(ValueError, match="Invalid date format"):
            validate_date("01-01-2025")

    def test_invalid_month(self) -> None:
        with pytest.raises(ValueError, match="Invalid date format"):
            validate_date("2025-13-01")

    def test_invalid_day(self) -> None:
        with pytest.raises(ValueError, match="Invalid date format"):
            validate_date("2025-02-30")

    def test_partial_date(self) -> None:
        with pytest.raises(ValueError, match="Invalid date format"):
            validate_date("2025-01")

    def test_empty_string(self) -> None:
        with pytest.raises(ValueError, match="Invalid date format"):
            validate_date("")

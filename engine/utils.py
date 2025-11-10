"""
Utility functions for the regelrecht engine
"""

from typing import Any
import math


def clean_nan_value(value: Any) -> Any:
    """
    Clean NaN values from pandas/numpy

    Args:
        value: Any value that might be NaN

    Returns:
        None if value is NaN, otherwise the value
    """
    if value is None:
        return None

    # Handle pandas/numpy NaN
    try:
        if math.isnan(value):
            return None
    except (TypeError, ValueError):
        pass

    return value


def resolve_variable_reference(value: Any, context: "RuleContext") -> Any:
    """
    Resolve a value that might be a variable reference ($VARIABLE)

    Args:
        value: Value to resolve (string starting with $ or literal value)
        context: Execution context

    Returns:
        Resolved value
    """
    if isinstance(value, str) and value.startswith("$"):
        variable_name = value[1:]
        return context._resolve_value(variable_name)
    return value


def format_amount(amount_eurocent: int) -> str:
    """
    Format an amount in eurocent as EUR string

    Args:
        amount_eurocent: Amount in eurocent (e.g., 209692)

    Returns:
        Formatted string (e.g., "€2,096.92")
    """
    euros = amount_eurocent / 100
    return f"€{euros:,.2f}"


def parse_amount(amount_str: str) -> int:
    """
    Parse amount string to eurocent

    Args:
        amount_str: Amount string like "2096.92" or "€2,096.92"

    Returns:
        Amount in eurocent
    """
    # Remove currency symbols and commas
    cleaned = amount_str.replace("€", "").replace(",", "").strip()

    # Parse as float and convert to cents
    euros = float(cleaned)
    return int(euros * 100)

"""
Logging configuration for the regelrecht engine
"""

import logging
import sys


def setup_logging(level=logging.INFO):
    """
    Configure logging for the engine

    Args:
        level: Logging level (default: INFO)
    """
    # Create logger
    logger = logging.getLogger("regelrecht")
    logger.setLevel(level)

    # Remove existing handlers
    logger.handlers = []

    # Create console handler
    handler = logging.StreamHandler(sys.stdout)
    handler.setLevel(level)

    # Create formatter
    formatter = logging.Formatter(
        "%(asctime)s - %(name)s - %(levelname)s - %(message)s"
    )
    handler.setFormatter(formatter)

    # Add handler to logger
    logger.addHandler(handler)

    return logger


# Create default logger
logger = logging.getLogger("regelrecht")

"""
Logging configuration for the regelrecht engine

Includes IndentLogger for hierarchical tree-style execution visualization.
"""

import io
import logging
import sys
from contextlib import contextmanager


class GlobalIndent:
    """Global indentation and tree state for hierarchical logging"""

    _level = 0
    _tree_chars_single = {
        "pipe": "│",
        "branch": "├──",
        "leaf": "└──",
        "space": " " * 3,
    }
    _tree_chars_double = {
        "pipe": "║",
        "branch": "║──",
        "leaf": "╚══",
        "space": " " * 3,
    }
    _active_branches: set[int] = set()
    _double_lines: set[int] = set()

    @classmethod
    def increase(cls, double_line: bool = False) -> None:
        """Increase indentation level"""
        cls._level += 1
        cls._active_branches.add(cls._level - 1)
        if double_line:
            cls._double_lines.add(cls._level - 1)

    @classmethod
    def decrease(cls) -> None:
        """Decrease indentation level"""
        if cls._level > 0:
            # No longer active - will show end corner
            cls._active_branches.discard(cls._level - 1)
            cls._double_lines.discard(cls._level - 1)
            cls._level -= 1

    @classmethod
    def reset(cls) -> None:
        """Reset indentation state (useful for tests)"""
        cls._level = 0
        cls._active_branches = set()
        cls._double_lines = set()

    @classmethod
    def get_indent(cls) -> str:
        """Get current indentation string with tree characters"""
        if cls._level == 0:
            return ""

        parts = []
        # For all levels except current, show pipe only if level is still active
        for i in range(cls._level - 1):
            if i in cls._active_branches:
                chars = (
                    cls._tree_chars_double
                    if i in cls._double_lines
                    else cls._tree_chars_single
                )
                parts.append(f"{chars['pipe']}   ")
            else:
                parts.append("    ")

        # For current level, use leaf if not active (end of block)
        chars = (
            cls._tree_chars_double
            if (cls._level - 1) in cls._double_lines
            else cls._tree_chars_single
        )
        is_end = (cls._level - 1) not in cls._active_branches
        parts.append(chars["leaf"] if is_end else chars["branch"])
        return "".join(parts)


class IndentLogger:
    """Logger wrapper that handles indentation using global state"""

    def __init__(self, base_logger: logging.Logger) -> None:
        self._logger = base_logger

    def debug(self, msg: str, *args, **kwargs) -> None:
        """Log debug message with indentation"""
        self._logger.debug(f"{self.indent}{msg}", *args, **kwargs)

    def info(self, msg: str, *args, **kwargs) -> None:
        """Log info message with indentation"""
        self._logger.info(f"{self.indent}{msg}", *args, **kwargs)

    def warning(self, msg: str, *args, **kwargs) -> None:
        """Log warning message with indentation"""
        self._logger.warning(f"{self.indent}{msg}", *args, **kwargs)

    def error(self, msg: str, *args, **kwargs) -> None:
        """Log error message with indentation"""
        self._logger.error(f"{self.indent}{msg}", *args, **kwargs)

    @property
    def indent(self) -> str:
        """Get current indentation string"""
        return GlobalIndent.get_indent()

    @contextmanager
    def indent_block(
        self, initial_message: str | None = None, double_line: bool = False
    ):
        """
        Context manager for handling indentation blocks

        Args:
            initial_message: Optional message to log at block start
            double_line: Use double-line characters for emphasis
        """
        if initial_message:
            self.debug(initial_message)
        GlobalIndent.increase(double_line)
        try:
            yield
        finally:
            GlobalIndent.decrease()


def setup_logging(level=logging.INFO):
    """
    Configure logging for the engine

    Args:
        level: Logging level (default: INFO)

    Returns:
        IndentLogger: Configured logger with indentation support
    """
    # Create logger
    base_logger = logging.getLogger("regelrecht")
    base_logger.setLevel(level)

    # Remove existing handlers
    base_logger.handlers = []

    # Create console handler with UTF-8 encoding (fixes Windows cp1252 issues)
    stream = io.TextIOWrapper(sys.stdout.buffer, encoding="utf-8", errors="replace")
    handler = logging.StreamHandler(stream)
    handler.setLevel(level)

    # Create formatter (simple format for tree-style output)
    formatter = logging.Formatter("%(levelname)8s %(message)s")
    handler.setFormatter(formatter)

    # Add handler to logger
    base_logger.addHandler(handler)

    return IndentLogger(base_logger)


# Create default logger with indentation support
logger = IndentLogger(logging.getLogger("regelrecht"))

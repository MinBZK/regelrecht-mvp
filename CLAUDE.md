# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**regelrecht-mvp** is an early-stage MVP project with two primary components:
- `engine/` - Core processing/business logic component
- `regulation/` - Regulatory compliance component

The repository is currently in initial setup phase with minimal code structure.

## Development Setup

This is a Python project managed with **uv** (https://github.com/astral-sh/uv).

### Initial Setup
```bash
# uv will automatically create a virtual environment and install dependencies
uv sync
```

### Common Commands
```bash
# Run Python scripts
uv run python script.py

# Run the main application
uv run python main.py

# Add a new dependency
uv add package-name

# Add a development dependency
uv add --dev package-name

# Run pre-commit hooks manually
uv run pre-commit run --all-files
```

### Pre-commit Hooks
This repository uses pre-commit hooks for code quality:
- **Ruff**: Fast Python linter and formatter
- **Standard hooks**: Trailing whitespace, end-of-file fixer, YAML checks, etc.

Hooks are automatically installed with `uv run pre-commit install` and run on every commit.

## Architecture Notes

The project is organized into two main directories representing distinct functional areas:
- **engine/** - Intended for core system logic and processing capabilities
- **regulation/** - Intended for regulatory compliance handling and validation

## Technology Stack

- **Language**: Python 3.12+
- **Package Manager**: uv
- **Code Quality**: Ruff (linting and formatting), pre-commit hooks

## Future Development

As this codebase grows, this CLAUDE.md should be updated to include:
- Architecture patterns and key design decisions
- Integration points between engine and regulation components
- Testing strategies and requirements
- API documentation and usage examples

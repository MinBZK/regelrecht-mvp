# Justfile voor regelrecht-mvp
# Gebruik: just <task>

# Default task - toon beschikbare tasks
default:
    @just --list

# Run pytest
test:
    uv run pytest

# Run behave BDD tests
behave:
    uv run behave

# Run alle tests (pytest + behave)
test-all: test behave

# Lint met ruff
lint:
    uv run ruff check .

# Format met ruff
format:
    uv run ruff format .

# Type check met ty
typecheck:
    uv run ty check

# YAML lint
yamllint:
    uv run yamllint regulation/

# Valideer YAML tegen schema
validate file:
    uv run python script/validate.py {{file}}

# Valideer alle regulation YAML files
validate-all:
    uv run python -c "import glob; import subprocess; [subprocess.run(['uv', 'run', 'python', 'script/validate.py', f]) for f in glob.glob('regulation/**/*.yaml', recursive=True)]"

# Alle checks (lint + typecheck)
check: lint typecheck

# Pre-commit hooks draaien
pre-commit:
    uv run pre-commit run --all-files

# Sync dependencies
sync:
    uv sync

# Setup feature file links (creates junction for behave to find features in regulation folder)
setup-links:
    #!/usr/bin/env bash
    if [[ "$OSTYPE" == "msys" || "$OSTYPE" == "cygwin" || "$OSTYPE" == "win32" ]]; then
        # Windows: create junction
        if [ ! -d "features/regulation" ]; then
            cmd //c "mklink /J features\\regulation regulation\\nl"
        fi
    else
        # Unix: create symlink
        if [ ! -L "features/regulation" ]; then
            ln -s ../regulation/nl features/regulation
        fi
    fi

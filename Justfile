# Justfile voor regelrecht-mvp
# Gebruik: just <task>

# Default task - toon beschikbare tasks
default:
    @just --list

# Run pytest
test:
    uv run pytest

# Run pytest with optional test filter (e.g. just test-filter test_article_builder)
test-filter filter="":
    uv run pytest {{ if filter != "" { "-k " + filter } else { "" } }} -v

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

# Download fresh XML fixtures from BWB repository (all fixtures)
download-harvester-fixtures:
    uv run python script/download_harvester_fixtures.py

# Download specific law fixture from BWB repository
download-harvester-fixture law:
    uv run python script/download_harvester_fixtures.py --law {{law}}

# Update harvester test fixtures (regenerate expected YAML from input XML)
update-harvester-fixtures:
    uv run python script/update_harvester_fixtures.py

# Download a law from BWB repository (harvester)
harvest bwb_id date="2025-01-01" *args="":
    uv run python -m harvester download {{bwb_id}} --date {{date}} {{args}}

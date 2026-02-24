# Justfile voor regelrecht-mvp
# Gebruik: just <task>

# Default task - toon beschikbare tasks
default:
    @just --list

# =============================================================================
# Rust quality checks
# =============================================================================

# Check Rust formatting (cargo fmt --check)
format:
    cargo fmt --all --check

# Run clippy lints on all packages
lint:
    cargo clippy --all-targets --all-features -- -D warnings

# Run cargo check on all packages
build-check:
    cargo check --all-targets --all-features

# =============================================================================
# Validation
# =============================================================================

# Validate regulation YAML files (all, or pass specific files)
validate *files:
    #!/usr/bin/env bash
    if [ -z "{{files}}" ]; then
        ./script/validate.sh
    else
        ./script/validate.sh {{files}}
    fi

# =============================================================================
# Tests
# =============================================================================

# Run Rust unit tests
test:
    cargo test --lib -p regelrecht-engine

# Run Rust BDD tests (cucumber-rs)
bdd:
    cargo test --test bdd -p regelrecht-engine --features bdd -- --nocapture

# Run all tests (unit + BDD)
test-all: test bdd

# =============================================================================
# Combined checks
# =============================================================================

# Run all quality checks (format + lint + check + validate + tests)
check: format lint build-check validate test-all

# =============================================================================
# Pre-commit hooks
# =============================================================================

# Pre-commit hooks draaien
pre-commit:
    pre-commit run --all-files

# =============================================================================
# Frontend development
# =============================================================================

# Start frontend dev server (Vite)
dev:
    cd frontend && npm run dev

# YAML lint (requires yamllint)
yamllint:
    yamllint regulation/

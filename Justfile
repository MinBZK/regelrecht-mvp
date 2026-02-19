# Justfile voor regelrecht-mvp
# Gebruik: just <task>

# Default task - toon beschikbare tasks
default:
    @just --list

# --- Quality checks ---

# Check Rust formatting
format:
    cd packages && cargo fmt --check --all

# Run clippy lints
lint:
    cd packages && cargo clippy --all-features -- -D warnings

# Run cargo check
build-check:
    cd packages && cargo check --all-features

# Validate regulation YAML files
validate *FILES:
    script/validate.sh {{FILES}}

# Run all quality checks (format + lint + check + validate + tests)
check: format lint build-check validate test-all

# --- Tests ---

# Run Rust unit and integration tests
test:
    cd packages/engine && cargo test --all-features

# Run Rust BDD tests
bdd:
    cd packages/engine && cargo test --test bdd -- --nocapture

# Run harvester tests
harvester-test:
    cd packages/harvester && cargo test

# Run all tests (engine + harvester)
test-all: test harvester-test

# --- Security ---

# Run security audit on all dependencies (vulnerabilities, licenses, sources)
audit:
    cargo deny check
    cd frontend && npm audit
    cd frontend && npx license-checker --production --failOn "GPL-2.0;GPL-3.0;AGPL-1.0;AGPL-3.0;SSPL-1.0;BUSL-1.1"

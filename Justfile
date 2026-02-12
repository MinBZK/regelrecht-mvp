# Justfile voor regelrecht-mvp
# Gebruik: just <task>

# Default task - toon beschikbare tasks
default:
    @just --list

# Run Rust unit tests
test:
    cd packages/engine && cargo test --lib

# Run Rust BDD tests
bdd:
    cd packages/engine && cargo test --test bdd -- --nocapture

# Run all Rust tests (unit + BDD + harvester)
test-all: test bdd harvester-test

# Run harvester tests
harvester-test:
    cd packages/harvester && cargo test

# Check harvester
harvester-check:
    cd packages/harvester && cargo check --all-features

# Harvester clippy
harvester-clippy:
    cd packages/harvester && cargo clippy --all-features -- -D warnings

# Harvester format check
harvester-fmt:
    cd packages/harvester && cargo fmt --check

# Validate regulation YAML files
validate *FILES:
    script/validate.sh {{FILES}}

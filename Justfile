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

# Run all Rust tests (unit + BDD)
test-all: test bdd

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

# Run pipeline tests (requires Docker for testcontainers)
pipeline-test:
    cd packages/pipeline && cargo test

# Check pipeline compilation (works offline, no DB needed)
pipeline-check:
    cd packages/pipeline && SQLX_OFFLINE=true cargo check

# Run harvest worker
harvest-worker:
    cd packages/pipeline && cargo run --bin regelrecht-harvest-worker

# Start local Postgres for development
db-up:
    docker compose up -d postgres

# Stop local Postgres
db-down:
    docker compose down

# Run database migrations (requires local Postgres)
db-migrate:
    cd packages/pipeline && cargo sqlx migrate run

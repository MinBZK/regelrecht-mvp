#!/usr/bin/env bash
# Validate regulation YAML files against the law schema.
#
# Usage:
#   script/validate.sh regulation/nl/wet/wet_op_de_zorgtoeslag/2025-01-01.yaml
#   script/validate.sh regulation/nl/**/*.yaml
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"

exec cargo run --manifest-path "$REPO_ROOT/packages/engine/Cargo.toml" --bin validate -- "$@"

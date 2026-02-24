#!/bin/bash
# Start the HighlightEditor server
#
# This script runs the existing Python server which already supports:
# - GET /api/regulations
# - GET /api/regulation/{id}  (with annotations)
# - POST /api/regulation/{id}/annotation
# - Serving static frontend files
#
# Once Rust build tools are available, you can use:
#   cd doc/experimenten/test5 && cargo run
#
# For now, use the Python server:

cd "$(dirname "$0")/../../.."
echo "Starting RegelRecht server on http://localhost:8000"
echo "Open http://localhost:8000/participatiewet.html to test the HighlightEditor"
uv run python server.py

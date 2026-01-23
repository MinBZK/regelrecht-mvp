#!/bin/bash
# Build script for WASM package
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ENGINE_DIR="$(dirname "$SCRIPT_DIR")"

echo "Building WASM package..."
cd "$ENGINE_DIR"

# Check required files exist
if [ ! -f README.md ]; then
    echo "Error: README.md not found"
    exit 1
fi
if [ ! -f LICENSE ]; then
    echo "Error: LICENSE not found"
    exit 1
fi

# Build with wasm-pack
wasm-pack build --target web --features wasm

# Verify critical build outputs
if [ ! -f pkg/regelrecht_engine_bg.wasm ]; then
    echo "Error: WASM binary not generated"
    exit 1
fi
if [ ! -f pkg/regelrecht_engine.js ]; then
    echo "Error: JS glue code not generated"
    exit 1
fi
if [ ! -f pkg/regelrecht_engine.d.ts ]; then
    echo "Error: TypeScript definitions not generated"
    exit 1
fi

# Copy additional files to pkg
cp README.md pkg/ || { echo "Failed to copy README.md"; exit 1; }
cp LICENSE pkg/ || { echo "Failed to copy LICENSE"; exit 1; }

# Update package.json to include README.md and LICENSE in files array
# Using node for cross-platform JSON manipulation
node -e "
const fs = require('fs');
const pkg = JSON.parse(fs.readFileSync('pkg/package.json', 'utf8'));
if (!pkg.files.includes('README.md')) pkg.files.push('README.md');
if (!pkg.files.includes('LICENSE')) pkg.files.push('LICENSE');
fs.writeFileSync('pkg/package.json', JSON.stringify(pkg, null, 2) + '\n');
console.log('Updated package.json files array');
"

echo ""
echo "WASM package built successfully!"
echo "Output: $ENGINE_DIR/pkg/"
ls -la pkg/

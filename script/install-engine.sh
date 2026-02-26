#!/usr/bin/env bash
# Install the regelrecht evaluate binary from GitHub releases.
#
# Usage:
#   ./script/install-engine.sh              # latest release
#   ./script/install-engine.sh engine-v0.1.0  # specific version
#   INSTALL_DIR=/usr/local/bin ./script/install-engine.sh  # custom install dir
#
# Environment variables:
#   INSTALL_DIR   - Directory to install the binary (default: /usr/local/bin)
#   GITHUB_TOKEN  - Optional token for private repos or rate limiting

set -euo pipefail

REPO="MinBZK/regelrecht-mvp"
VERSION="${1:-latest}"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"

# Detect platform
OS="$(uname -s)"
ARCH="$(uname -m)"

case "${OS}-${ARCH}" in
    Linux-x86_64)   ASSET="evaluate-linux-amd64" ;;
    Darwin-x86_64)  ASSET="evaluate-macos-amd64" ;;
    Darwin-arm64)   ASSET="evaluate-macos-arm64" ;;
    *)
        echo "Unsupported platform: ${OS}-${ARCH}" >&2
        exit 1
        ;;
esac

# Build download URL
if [ "${VERSION}" = "latest" ]; then
    URL="https://github.com/${REPO}/releases/latest/download/${ASSET}"
else
    URL="https://github.com/${REPO}/releases/download/${VERSION}/${ASSET}"
fi

echo "Downloading ${ASSET} (${VERSION})..."

CURL_OPTS=(-fSL --retry 3)
if [ -n "${GITHUB_TOKEN:-}" ]; then
    CURL_OPTS+=(-H "Authorization: token ${GITHUB_TOKEN}")
fi

mkdir -p "${INSTALL_DIR}"
curl "${CURL_OPTS[@]}" -o "${INSTALL_DIR}/evaluate" "${URL}"
chmod +x "${INSTALL_DIR}/evaluate"

echo "Installed: ${INSTALL_DIR}/evaluate"

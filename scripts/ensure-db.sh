#!/usr/bin/env bash
# Ensure the .maximous directory, binary, and correct version exist.
# Called on SessionStart to prepare everything before the MCP server starts.
set -euo pipefail

DB_DIR="${MAXIMOUS_DB_DIR:-.maximous}"
mkdir -p "$DB_DIR"

# Read expected version from plugin.json
PLUGIN_JSON="${CLAUDE_PLUGIN_ROOT:-.}/.claude-plugin/plugin.json"
EXPECTED_VERSION=""
if [ -f "$PLUGIN_JSON" ]; then
  # Extract version without jq dependency
  EXPECTED_VERSION=$(sed -n 's/.*"version"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' "$PLUGIN_JSON" | head -1)
fi

# Find the installed binary
MAXIMOUS_BIN=""
if command -v maximous &>/dev/null; then
  MAXIMOUS_BIN="$(command -v maximous)"
elif [ -x "${HOME}/.cargo/bin/maximous" ]; then
  MAXIMOUS_BIN="${HOME}/.cargo/bin/maximous"
elif [ -x "${CLAUDE_PLUGIN_ROOT:-}/bin/maximous" ]; then
  MAXIMOUS_BIN="${CLAUDE_PLUGIN_ROOT}/bin/maximous"
fi

# Check version if binary exists
NEEDS_UPDATE=false
if [ -n "$MAXIMOUS_BIN" ]; then
  if [ -n "$EXPECTED_VERSION" ]; then
    INSTALLED_VERSION=$("$MAXIMOUS_BIN" --version 2>/dev/null | sed 's/[^0-9.]//g' || echo "unknown")
    if [ "$INSTALLED_VERSION" != "$EXPECTED_VERSION" ]; then
      echo "maximous: installed v${INSTALLED_VERSION}, expected v${EXPECTED_VERSION} — updating..." >&2
      NEEDS_UPDATE=true
    else
      exit 0
    fi
  else
    exit 0
  fi
else
  NEEDS_UPDATE=true
fi

# Download and install the correct version
REPO="awesome-lab/maximous"
INSTALL_DIR="${HOME}/.cargo/bin"
OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

case "$ARCH" in
  x86_64|amd64) ARCH="x86_64" ;;
  arm64|aarch64) ARCH="arm64" ;;
  *) echo "maximous: unsupported architecture $ARCH — install manually with: cargo install --git https://github.com/${REPO}" >&2; exit 0 ;;
esac

case "$OS" in
  darwin) OS="darwin" ;;
  linux)  OS="linux" ;;
  *)      echo "maximous: unsupported OS $OS — install manually with: cargo install --git https://github.com/${REPO}" >&2; exit 0 ;;
esac

TARGET="${OS}-${ARCH}"

# Download specific version if known, otherwise latest
if [ -n "$EXPECTED_VERSION" ]; then
  DOWNLOAD_URL="https://github.com/${REPO}/releases/download/v${EXPECTED_VERSION}/maximous-${TARGET}.tar.gz"
else
  DOWNLOAD_URL="https://github.com/${REPO}/releases/latest/download/maximous-${TARGET}.tar.gz"
fi

if [ "$NEEDS_UPDATE" = true ] && [ -n "$MAXIMOUS_BIN" ]; then
  echo "maximous: updating binary from GitHub Releases..." >&2
else
  echo "maximous: binary not found, downloading from GitHub Releases..." >&2
fi

TMPDIR="$(mktemp -d)"
trap 'rm -rf "$TMPDIR"' EXIT

if curl -fsSL "$DOWNLOAD_URL" -o "${TMPDIR}/maximous.tar.gz" 2>/dev/null; then
  tar -xzf "${TMPDIR}/maximous.tar.gz" -C "$TMPDIR"
  mkdir -p "$INSTALL_DIR"
  mv "${TMPDIR}/maximous" "${INSTALL_DIR}/maximous"
  chmod +x "${INSTALL_DIR}/maximous"
  echo "maximous: installed v${EXPECTED_VERSION:-latest} to ${INSTALL_DIR}/maximous" >&2
else
  echo "maximous: download failed — install manually with: cargo install --git https://github.com/${REPO}" >&2
fi

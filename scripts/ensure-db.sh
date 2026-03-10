#!/usr/bin/env bash
# Ensure the .maximous directory exists for the project database.
# Called on SessionStart to prepare the DB path.
set -euo pipefail

DB_DIR="${MAXIMOUS_DB_DIR:-.maximous}"
mkdir -p "$DB_DIR"

# Check if maximous binary is available
if ! command -v maximous &>/dev/null && [ ! -x "${CLAUDE_PLUGIN_ROOT:-}/bin/maximous" ] && [ ! -x "${HOME}/.cargo/bin/maximous" ]; then
  echo "maximous binary not found. Install with: cargo install --git https://github.com/laurentlouk/maximous" >&2
fi

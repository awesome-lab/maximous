#!/usr/bin/env bash
# Launcher script for maximous MCP server.
# Finds the binary in known locations and starts it with project-local DB.
set -euo pipefail

# Determine database path (project-local)
DB_PATH="${MAXIMOUS_DB:-.maximous/brain.db}"

# Search order for the binary
CANDIDATES=(
  "${CLAUDE_PLUGIN_ROOT:-}/bin/maximous"
  "${HOME}/.cargo/bin/maximous"
  "$(command -v maximous 2>/dev/null || true)"
)

BINARY=""
for candidate in "${CANDIDATES[@]}"; do
  if [ -n "$candidate" ] && [ -x "$candidate" ]; then
    BINARY="$candidate"
    break
  fi
done

if [ -z "$BINARY" ]; then
  echo "ERROR: maximous binary not found. Install with:" >&2
  echo "  cargo install --path \${CLAUDE_PLUGIN_ROOT}" >&2
  echo "  or download from GitHub Releases" >&2
  exit 1
fi

exec "$BINARY" --db "$DB_PATH"

# Maximous

SQLite brain for Claude Code — multi-agent orchestration, session continuity, web dashboard.

## Release Checklist

When preparing a new version, **all three files must be updated together**:

1. `Cargo.toml` — `version = "X.Y.Z"` (used by CI to tag the GitHub Release)
2. `.claude-plugin/plugin.json` — `"version": "X.Y.Z"` (used by `ensure-db.sh` to auto-update the binary)
3. `web/index.html` — `<span class="version">vX.Y.Z</span>` (displayed in the dashboard UI)

Then commit and push to `main`. CI will automatically:
- Run tests and clippy
- Build binaries for macOS (x86_64 + arm64) and Linux (x86_64 + arm64)
- Create a GitHub Release tagged `vX.Y.Z` with the binaries

The `ensure-db.sh` SessionStart hook compares the installed binary version against `plugin.json` and auto-downloads the matching release if they differ. So once the release is published, any machine running the plugin will auto-update on next session start.

## Architecture

- **Rust binary** (`src/`) — MCP server + web dashboard server, compiled with `rust-embed` (dashboard files in `web/` are embedded at build time)
- **Plugin config** (`.claude-plugin/plugin.json`) — declares the MCP server command
- **Hooks** (`hooks/hooks.json`) — SessionStart runs `ensure-db.sh`, session continuity prompts
- **Commands** (`commands/`) — slash commands like `/dashboard`
- **Skills** (`skills/`) — orchestration, coordination, memory patterns

## Development

```bash
cargo build          # build
cargo test           # run tests
cargo clippy         # lint
maximous dashboard --db .maximous/brain.db  # run dashboard locally
```

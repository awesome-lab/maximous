# Maximous: Lightweight SQLite Brain for Multi-Agent Orchestration

## Overview

A single Rust binary that acts as an MCP server over stdio, providing SQLite-backed shared memory, messaging, and task coordination for Claude Code agents. Designed for multi-agent workflows where sub-agents, team agents, and parallel agents need fast communication and shared state.

## Architecture

- **Runtime:** Single Rust binary, no external dependencies
- **Communication:** MCP server over stdio (JSON-RPC)
- **Storage:** Embedded SQLite via `rusqlite` with WAL mode
- **DB location:** `.maximous/brain.db` (project-local)
- **Concurrency model:** Each agent spawns its own MCP process; all processes share one SQLite file via WAL (concurrent reads, serialized writes)

### Key Crates

- `rusqlite` (bundled feature) -- SQLite with no system dependency
- `serde` / `serde_json` -- JSON serialization
- `tokio` -- async runtime for stdio MCP loop
- `clap` -- CLI args (db path, log level)

## Schema (6 tables + 1 change log)

```sql
-- Shared knowledge store
CREATE TABLE memory (
    namespace TEXT NOT NULL,
    key TEXT NOT NULL,
    value TEXT NOT NULL,
    ttl_seconds INTEGER,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    PRIMARY KEY (namespace, key)
);

-- Inter-agent messages
CREATE TABLE messages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    channel TEXT NOT NULL,
    sender TEXT NOT NULL,
    priority INTEGER DEFAULT 2,   -- 0=critical, 1=high, 2=normal, 3=low
    content TEXT NOT NULL,
    acknowledged INTEGER DEFAULT 0,
    created_at INTEGER NOT NULL
);

-- Task coordination
CREATE TABLE tasks (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    priority INTEGER DEFAULT 2,
    assigned_to TEXT,
    dependencies TEXT,             -- JSON array of task IDs
    result TEXT,                   -- JSON
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

-- Agent registry
CREATE TABLE agents (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'idle',
    capabilities TEXT,             -- JSON array
    metadata TEXT,                 -- JSON
    last_heartbeat INTEGER NOT NULL
);

-- Observation / event log
CREATE TABLE changes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    table_name TEXT NOT NULL,
    row_id TEXT NOT NULL,
    action TEXT NOT NULL,          -- insert/update/delete
    summary TEXT,                  -- JSON snippet
    created_at INTEGER NOT NULL
);

-- Key-value config
CREATE TABLE config (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
```

SQLite triggers on `memory`, `messages`, `tasks`, and `agents` auto-populate the `changes` table on every INSERT/UPDATE/DELETE.

## MCP Tools (14 tools)

### Memory (shared brain)

| Tool | Description |
|---|---|
| `memory_set` | Write (namespace, key, value, ttl?) |
| `memory_get` | Read by namespace+key, or list keys in a namespace |
| `memory_search` | Full-text search across values with optional namespace filter |
| `memory_delete` | Remove a key or expire all stale entries |

### Messages (communication bus)

| Tool | Description |
|---|---|
| `message_send` | Send to a channel with priority |
| `message_read` | Read messages from a channel, filter unacknowledged |
| `message_ack` | Acknowledge a message by ID |

### Tasks (coordination board)

| Tool | Description |
|---|---|
| `task_create` | Create task with dependencies (JSON array of task IDs) |
| `task_update` | Update status, assignment, result |
| `task_list` | List tasks filtered by status/assignee, ordered by priority |

### Observation (the glue)

| Tool | Description |
|---|---|
| `poll_changes` | Returns all changes since a given change ID |

### Agent Registry

| Tool | Description |
|---|---|
| `agent_register` | Register self with capabilities |
| `agent_heartbeat` | Update heartbeat + status |
| `agent_list` | List active agents (heartbeat within last 60s) |

### Key Behaviors

- `task_update` to `ready` auto-checks dependencies -- only allows if all deps are `done`
- `memory_set` with TTL uses lazy expiration (cleanup on next read, no background thread)
- `poll_changes` is the single observation tool -- agents watch all tables through it
- All tools return `{ok: true, data: ...}` or `{ok: false, error: ...}`

## Project Structure

```
maximous/
├── Cargo.toml
├── src/
│   ├── main.rs              -- CLI entry + MCP stdio loop
│   ├── db.rs                -- SQLite setup, migrations, triggers
│   ├── mcp.rs               -- JSON-RPC dispatcher
│   ├── tools/
│   │   ├── memory.rs
│   │   ├── messages.rs
│   │   ├── tasks.rs
│   │   ├── agents.rs
│   │   └── changes.rs
│   └── schema.sql           -- embedded via include_str!
├── plugin/
│   ├── plugin.json           -- Claude Code plugin manifest
│   └── .mcp.json             -- MCP server declaration
└── README.md
```

## Plugin Integration

### Plugin Manifest

```json
{
  "name": "maximous",
  "description": "Lightweight SQLite brain for multi-agent orchestration",
  "mcp_servers": {
    "maximous": {
      "command": "maximous",
      "args": ["--db", ".maximous/brain.db"]
    }
  }
}
```

### Concurrency Model

```
Agent A (subagent)  --stdio-->  maximous process A  --WAL--+
Agent B (subagent)  --stdio-->  maximous process B  --WAL--+-->  brain.db
Agent C (team)      --stdio-->  maximous process C  --WAL--+
```

Each agent gets its own MCP process. WAL mode handles concurrent reads with no blocking, serialized writes with automatic locking, and no corruption.

## Typical Multi-Agent Flow

1. Orchestrator creates tasks with dependencies
2. Agent A picks up a task with no dependencies, runs it, stores results in memory
3. Other agents poll changes, see the task is done
4. Dependent tasks become ready, agents pick them up in parallel
5. Agents read shared memory for upstream results
6. Agents communicate via message channels if needed

## Design Decisions

| Decision | Rationale |
|---|---|
| Rust over TypeScript | Single binary, no runtime, sub-ms startup, memory safe |
| stdio MCP over HTTP | Native Claude Code integration, no networking, no auth |
| SQLite WAL over in-memory | Crash recovery, multi-process safe, persisted state |
| Triggers over polling raw tables | Single changes table, efficient integer-based cursors |
| Lazy TTL expiration | No background threads, keeps binary simple |
| ~14 tools over 50+ | Each tool is essential, no bloat |

## Inspiration

Key ideas borrowed from [ruflo](https://github.com/ruvnet/ruflo):
- Namespaced memory with TTL
- Priority message queue with channels
- Task dependency graphs
- Change log / observation pattern
- Agent capability registry

What we deliberately left out:
- Neural patterns, consensus algorithms, topology routing (over-engineered)
- In-memory event bus (SQLite WAL replaces this)
- Background optimization loops (lazy expiration instead)
- 100+ skills/agents definitions (out of scope -- this is infrastructure only)

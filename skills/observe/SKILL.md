---
name: observe
description: This skill should be used when an agent needs to "watch for changes", "poll for updates", "observe task completion", "react to events", "monitor agent activity", "wait for another agent", or needs to implement the observation pattern using maximous poll_changes tool.
---

# Observation Pattern

Watch for state changes across the entire maximous database using `poll_changes`. Every INSERT, UPDATE, and DELETE on memory, messages, tasks, and agents is automatically recorded by SQLite triggers into a changes table.

## How It Works

```
poll_changes(since_id=0, limit=100)
```

Returns changes ordered by ID:
```json
{
  "changes": [
    {"id": 1, "table_name": "tasks", "row_id": "abc-123", "action": "insert", "summary": "{\"title\":\"Parse API\",\"status\":\"pending\"}"},
    {"id": 2, "table_name": "tasks", "row_id": "abc-123", "action": "update", "summary": "{\"title\":\"Parse API\",\"status\":\"running\"}"},
    {"id": 3, "table_name": "memory", "row_id": "task-results:abc-123", "action": "insert", "summary": "{\"namespace\":\"task-results\",\"key\":\"abc-123\"}"}
  ]
}
```

## Cursor-Based Polling

Track the last seen change ID to only get new events:

1. First poll: `poll_changes(since_id=0)` — get everything
2. Note the highest `id` in the response (e.g., `42`)
3. Next poll: `poll_changes(since_id=42)` — only new changes

This is an efficient integer comparison — fast regardless of table size.

## Filtering by Table

Focus on a specific domain:
```
poll_changes(since_id=0, table_name="tasks")     # only task changes
poll_changes(since_id=0, table_name="messages")   # only message changes
poll_changes(since_id=0, table_name="agents")     # only agent changes
poll_changes(since_id=0, table_name="memory")     # only memory changes
```

## Pattern: Wait for Dependency

1. Create tasks with dependencies
2. Poll changes filtered to tasks table
3. When a change shows `"action": "update"` with `"status": "done"` for the dependency, the dependent task can proceed
4. Set dependent task to `ready` (maximous validates dependencies automatically)

## Pattern: React to Messages

1. Poll changes filtered to messages table
2. When a new message insert appears, read the full message
3. Process and acknowledge it

## Summary Field

The `summary` field is a JSON snippet from the trigger — it contains key fields but not the full row. Use it for quick filtering, then read the full record if needed.

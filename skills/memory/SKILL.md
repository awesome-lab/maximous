---
name: memory
description: This skill should be used when agents need to "store shared data", "read shared memory", "cache results", "share knowledge between agents", "use namespaced storage", "set TTL on data", or need a persistent key-value store through maximous memory tools.
---

# Shared Memory

Store and retrieve data between agents using maximous namespaced key-value memory. All agents sharing the same database file can read and write to the same memory.

## Namespaces

Organize data with namespaces to avoid key collisions:

- `task-results` — output from completed tasks
- `config` — shared configuration
- `cache` — temporary computed data (use TTL)
- `agent-state` — per-agent state snapshots
- `shared` — general-purpose shared data

## Writing

```
memory_set(
  namespace="task-results",
  key="parse-api",
  value="{\"endpoints\":[\"/users\",\"/items\"]}",
  ttl_seconds=3600
)
```

- Values are JSON strings
- `ttl_seconds` is optional — omit for permanent storage
- Writing to an existing key overwrites it (upsert)

## Reading

Get a specific key:
```
memory_get(namespace="task-results", key="parse-api")
```

List all keys in a namespace:
```
memory_get(namespace="task-results")
```
Returns `{"keys": [{"key": "parse-api", "updated_at": 1710000000}, ...]}`.

## Searching

Search across all values (or within a namespace):
```
memory_search(query="endpoints", namespace="task-results")
```
Uses SQL LIKE matching — finds any value containing the query string.

## TTL and Expiry

- Expired entries are cleaned up lazily on the next `memory_get` for that namespace
- No background threads — cleanup happens on read
- Set `ttl_seconds=0` for immediate expiry on next read
- Omit `ttl_seconds` for data that should persist indefinitely

## Deleting

Delete a specific key:
```
memory_delete(namespace="cache", key="old-result")
```

Expire all stale entries in a namespace:
```
memory_delete(namespace="cache")
```

## Pattern: Upstream Data Sharing

1. Agent A completes a task, stores result: `memory_set(namespace="task-results", key=task_id, value=result_json)`
2. Agent B polls changes, sees task is done
3. Agent B reads upstream result: `memory_get(namespace="task-results", key=task_id)`
4. Agent B uses the data without re-computing it

---
name: cleanup
description: This skill should be used when the user asks to "clean up maximous", "expire old data", "prune stale agents", "clean messages", "reset maximous state", or wants to maintain the maximous database by removing old data.
---

# Database Cleanup

Maintain the maximous database by expiring old data and removing stale entries.

## Expire Stale Memory

Remove all entries with expired TTL in a namespace:
```
memory_delete(namespace="cache")
memory_delete(namespace="task-results")
```

This only deletes entries where `created_at + ttl_seconds < now`. Entries without TTL are preserved.

## Check for Stale Agents

List all agents including stale ones:
```
agent_list(include_stale=true)
```

Agents with `last_heartbeat` older than 60 seconds are considered stale. Their assigned tasks may need reassignment.

## Acknowledge Old Messages

Read and acknowledge processed messages to keep channels clean:
```
message_read(channel="orchestration", unacknowledged_only=true)
message_ack(id=1)
message_ack(id=2)
```

## Full Reset

To completely reset the database, delete the `.maximous/brain.db` file. A fresh database is created automatically on next startup.

```bash
rm -f .maximous/brain.db
```

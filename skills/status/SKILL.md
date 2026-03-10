---
name: status
description: This skill should be used when the user asks for "maximous status", "show agent status", "list active tasks", "what's happening", "show current state", or wants a quick overview of the current maximous database state.
---

# Status Check

Get a quick overview of the current maximous state by querying agents, tasks, and recent changes.

## Quick Status

Run these three calls to get a full picture:

1. **Active agents**: `agent_list()` — who is online (heartbeat within 60s)
2. **Task board**: `task_list()` — all tasks sorted by priority
3. **Recent activity**: `poll_changes(since_id=0, limit=20)` — last 20 events

## Filtered Views

- Running tasks only: `task_list(status="running")`
- Blocked tasks: `task_list(status="pending")` — these are waiting for dependencies
- Agent's workload: `task_list(assigned_to="agent-id")`
- Unread messages: `message_read(channel="team", unacknowledged_only=true)`

## Presenting Status

Format the output as a summary:

```
Agents: 3 active (parser, builder, tester)
Tasks:  2 running, 1 ready, 3 pending, 5 done
Recent: parser completed "Parse API" 30s ago
        builder started "Build endpoints" 10s ago
```

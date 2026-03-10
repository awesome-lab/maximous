---
name: communicate
description: This skill should be used when agents need to "send a message", "read messages", "communicate with other agents", "ask a question to another agent", "use message channels", or need to establish inter-agent communication through maximous message tools.
---

# Agent Communication

Send and receive messages between agents using maximous priority message channels. Messages persist in SQLite and survive crashes.

## Channels

Channels are arbitrary strings. Use conventions to organize communication:

- `team` or `team-<name>` — team-wide broadcasts
- `agent-<id>` — direct messages to a specific agent
- `task-<id>` — discussion about a specific task
- `system` — system-level notifications

## Sending Messages

```
message_send(
  channel="team-backend",
  sender="agent-a",
  content="{\"type\":\"question\",\"text\":\"REST or GraphQL?\"}",
  priority=1
)
```

Content is a JSON string. Structure it with a `type` field for easy filtering:
- `{"type": "question", "text": "..."}` — asking for input
- `{"type": "answer", "text": "..."}` — responding
- `{"type": "status", "text": "..."}` — progress update
- `{"type": "alert", "text": "..."}` — something needs attention

## Reading Messages

```
message_read(channel="team-backend", unacknowledged_only=true, limit=10)
```

Messages return ordered by priority (critical first), then by time.

## Acknowledgment

After processing a message, acknowledge it so other agents reading the same channel know it's been handled:

```
message_ack(id=42)
```

Use `unacknowledged_only=true` when reading to skip already-handled messages.

## Priority Levels

- `0` = critical — urgent, handle immediately
- `1` = high — important but not blocking
- `2` = normal (default)
- `3` = low — informational

## Pattern: Request-Response

1. Agent A sends a question on a shared channel
2. Agent B reads unacknowledged messages, sees the question
3. Agent B acknowledges the question, sends a response on the same channel
4. Agent A reads the response

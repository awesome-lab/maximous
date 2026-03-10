---
name: coordinate
description: This skill should be used when an agent needs to "coordinate tasks", "pick up work", "check task dependencies", "create a task graph", "manage task lifecycle", or is participating in a multi-agent workflow that involves task assignment and dependency tracking through maximous.
---

# Task Coordination

Coordinate work between agents using maximous task tools. Tasks follow a lifecycle: `pending` -> `ready` -> `running` -> `done`/`failed`, with dependency checking enforced by the database.

## Workflow

### As an Orchestrator (creating work)

1. Create tasks with dependencies to form a task graph:
   ```
   task_create(title="Parse API spec", priority=1)              -> id: "abc-123"
   task_create(title="Build UI", dependencies=["abc-123"])      -> id: "def-456"
   task_create(title="Write tests", dependencies=["abc-123"])   -> id: "ghi-789"
   ```

2. Tasks with no dependencies start as `pending`. Tasks with dependencies cannot move to `ready` until all dependencies are `done`.

### As a Worker (executing work)

1. Register as an agent: `agent_register(id="my-id", name="Worker", capabilities=["coding"])`
2. List available tasks: `task_list(status="pending")` or `task_list(status="ready")`
3. Claim a task: `task_update(id="abc-123", status="running", assigned_to="my-id")`
4. Store results in shared memory when done: `memory_set(namespace="task-results", key="abc-123", value="{...}")`
5. Mark complete: `task_update(id="abc-123", status="done", result="{\"success\":true}")`
6. Send heartbeat periodically: `agent_heartbeat(id="my-id", status="active")`

### Monitoring Progress

Poll changes to watch for task completions without repeatedly listing all tasks:
```
poll_changes(since_id=0, table_name="tasks")
```
Track the returned `since_id` to only get new changes on subsequent polls.

## Priority Levels

- `0` = critical (run first)
- `1` = high
- `2` = normal (default)
- `3` = low

Tasks are listed in priority order (lowest number first).

## Error Handling

- If a task fails, set `status="failed"` with a result explaining why
- Dependent tasks will remain blocked (cannot move to `ready`)
- The orchestrator can create replacement tasks or adjust the graph

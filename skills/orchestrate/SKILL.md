---
name: orchestrate
description: This skill should be used when setting up a "multi-agent workflow", "orchestrating agents", "creating a task graph for parallel agents", "coordinating sub-agents", "planning agent work", or when an orchestrator agent needs to design and manage a complete multi-agent execution plan through maximous.
---

# Multi-Agent Orchestration

Design and manage complete multi-agent workflows using maximous. This skill is for the orchestrator — the agent that creates the plan, assigns work, and monitors progress.

## Setup Workflow

### 1. Register All Agents

```
agent_register(id="orchestrator", name="Orchestrator", capabilities=["planning", "coordination"])
agent_register(id="parser", name="Parser Agent", capabilities=["parsing", "analysis"])
agent_register(id="builder", name="Builder Agent", capabilities=["coding", "testing"])
```

### 2. Create Task Graph

Build a dependency graph where independent tasks run in parallel:

```
task_create(title="Parse API spec", priority=1)                           -> id: "t1"
task_create(title="Parse DB schema", priority=1)                          -> id: "t2"
task_create(title="Build API endpoints", dependencies=["t1", "t2"])       -> id: "t3"
task_create(title="Build UI components", dependencies=["t1"])             -> id: "t4"
task_create(title="Write integration tests", dependencies=["t3", "t4"])   -> id: "t5"
```

This creates: `t1` and `t2` run in parallel, `t3` waits for both, `t4` waits for `t1` only, `t5` waits for `t3` and `t4`.

### 3. Monitor Progress

Use the observation pattern to watch for completions:

```
poll_changes(since_id=0, table_name="tasks")
```

Track `since_id` between polls. When a task moves to `done`, check if dependent tasks can now move to `ready`.

### 4. Coordinate Communication

Set up channels for team communication:
- Broadcast status updates on `team` channel
- Use `task-<id>` channels for task-specific discussion
- Monitor `system` channel for errors

## Parallel Agent Pattern

For Claude Code sub-agents working in parallel:

1. Each sub-agent gets its own MCP server process (Claude Code handles this)
2. All processes share the same `brain.db` via SQLite WAL
3. Sub-agents register themselves, pick up tasks, store results
4. The orchestrator monitors via `poll_changes` without blocking

## Agent Health Monitoring

```
agent_list()  # returns agents with heartbeat in last 60 seconds
agent_list(include_stale=true)  # includes inactive agents
```

If an agent goes stale (no heartbeat for 60+ seconds), its assigned tasks may need reassignment.

## Completion Check

When all tasks are `done`:
1. Collect results from memory: `memory_get(namespace="task-results")`
2. Send summary to team channel
3. Clean up if needed: `memory_delete(namespace="cache")`

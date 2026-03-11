---
name: teams
description: This skill should be used when agents need to "define agents", "create teams", "list teams", "manage agent registry", "configure team members", or want to set up reusable agent definitions and team compositions for orchestration.
---

# Agent Registry & Teams

Define reusable agent configurations and group them into teams for fast orchestration.

## Agent Definitions

Create agents once, reuse everywhere:

```
agent_define(
  id="frontend-dev",
  name="Frontend Developer",
  capabilities=["typescript", "react", "tailwind"],
  model="sonnet",
  prompt_hint="Implement components following project conventions"
)
```

List all definitions:
```
agent_catalog()
```

Remove an agent:
```
agent_remove(id="frontend-dev")
```

## Teams

Group agents into teams:
```
team_create(
  name="frontend-squad",
  description="Build frontend features",
  members=[
    {"agent_id": "frontend-dev", "role": "implementer"},
    {"agent_id": "test-writer", "role": "tester"},
    {"agent_id": "code-reviewer", "role": "reviewer"}
  ]
)
```

Same agent can belong to multiple teams.

List teams with members:
```
team_list()
```

Delete a team:
```
team_delete(name="frontend-squad")
```

## Using Teams for Orchestration

When launching work, specify the team name. The orchestrator reads the full team config (agent definitions, roles, capabilities, models) to spawn the right subagents automatically.

---
name: dashboard
description: Open the maximous web dashboard in your browser
allowed_tools: ["Bash", "Agent", "mcp__plugin_maximous_maximous__launch_list", "mcp__plugin_maximous_maximous__launch_update", "WebFetch"]
---

Launch the maximous web dashboard AND start the launch listener. Two steps:

## Step 1: Start the dashboard server

Kill any existing dashboard process, then start a new one in the background using `run_in_background`:

```bash
lsof -ti:8375 | xargs kill 2>/dev/null; sleep 0.5; maximous dashboard --db .maximous/brain.db
```

Use the Bash tool with `run_in_background: true` so the server runs without blocking the conversation.

After launching, tell the user the dashboard is opening at http://127.0.0.1:8375.

## Step 2: Start the launch listener loop

After the dashboard is running, begin polling for pending launches every 30 seconds. This runs as a continuous loop in the conversation:

1. Call `launch_list(status="pending")`
2. For each pending launch found:
   a. Call `launch_update(id=..., status="running")` to claim it
   b. Dispatch an `Agent` with:
      - `isolation: "worktree"` — isolated git worktree
      - `run_in_background: true` — non-blocking
      - `mode: "auto"` — autonomous execution
      - A prompt containing: team name, members, ticket title/URL, branch, launch ID, and instructions to use `/maximous:orchestrate`
3. After processing, wait 30 seconds then poll again
4. If no pending launches, just wait and poll again silently

Tell the user: "Dashboard is running and listening for launches. Queue work from the Tickets page — I'll pick it up automatically."

Use the `/loop 30s` pattern: after each poll cycle, use Bash `sleep 30` then poll again. Keep looping until the user stops you.

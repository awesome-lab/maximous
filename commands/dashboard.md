---
name: dashboard
description: Open the maximous web dashboard in your browser
allowed_tools: ["Bash", "Agent", "mcp__plugin_maximous_maximous__launch_wait", "mcp__plugin_maximous_maximous__launch_update", "WebFetch"]
---

Launch the maximous web dashboard AND start the launch listener. Two steps:

## Step 1: Start the dashboard server

Kill any existing dashboard process, then start a new one in the background using `run_in_background`:

```bash
lsof -ti:8375 | xargs kill 2>/dev/null; sleep 0.5; maximous dashboard --db .maximous/brain.db
```

Use the Bash tool with `run_in_background: true` so the server runs without blocking the conversation.

After launching, tell the user the dashboard is opening at http://127.0.0.1:8375.

## Step 2: Start the launch listener

After the dashboard is running, use server-push via `launch_wait` to listen for launches. No polling or sleeping needed.

1. Call `launch_wait(timeout=120)` — this blocks server-side until a pending launch appears
2. Save the `cursor` from the response for subsequent calls
3. For each pending launch returned:
   a. Call `POST http://localhost:8375/api/launches/{id}/execute` via WebFetch to claim it and get full context
   b. Dispatch an `Agent` with:
      - `isolation: "worktree"` — isolated git worktree
      - `run_in_background: true` — non-blocking
      - `mode: "auto"` — autonomous execution
      - A prompt containing: team name, members, ticket title/URL, branch, launch ID, and instructions to use `/maximous:orchestrate`
4. Call `launch_wait(timeout=120, since_id=<cursor>)` again to wait for the next launch
5. On `timed_out: true`, just call `launch_wait` again with the same cursor

Tell the user: "Dashboard is running and listening for launches. Queue work from the Tickets page — I'll pick it up automatically."

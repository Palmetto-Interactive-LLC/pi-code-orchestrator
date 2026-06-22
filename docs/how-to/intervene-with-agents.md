# How to Intervene with Agents

Pause automation, take manual control, recover an agent, or send a note through the Temporal-backed human-control path.

## Find an Agent ID

```bash
lantern status
```

Or query the local projection:

```bash
sqlite3 ~/.lantern/data/relay/lantern.db \
  "SELECT id, role, status FROM agents WHERE session_id='myproject-1';"
```

Agent IDs follow the pattern `agent-{name}-{role}-{number}`.

## Pause an Agent

Blocks automated delivery until resumed:

```bash
lantern pause agent-myproject-ai-1
```

## Resume an Agent

```bash
lantern resume agent-myproject-ai-1
```

## Take Over an Agent

Human control should be recorded through the human-control workflow so other runtime decisions see it:

```bash
lantern takeover agent-myproject-ai-1
```

Work directly in the agent's iTerm pane while the takeover is active. When finished:

```bash
lantern release agent-myproject-ai-1
```

## Recover a Degraded Agent

Recovery should advance workflow state and runner readiness instead of falling back to direct terminal injection:

```bash
lantern recover agent-myproject-ai-1
```

The agent must re-establish runner and MCP readiness before new runtime delivery is considered safe.

## Send a Human Note

```bash
lantern note agent-myproject-ai-1 Check the failing test in src/db/queries.rs
```

Notes should be represented as human-control/runtime messages and delivered through the same Temporal-backed path as other control operations.

## When to Use Which Command

| Situation | Command |
|-----------|---------|
| Temporarily stop automation | `pause` / `resume` |
| Work directly in the pane | `takeover` / `release` |
| Runner, MCP, or execution window is degraded | `recover` |
| Send a message without taking over | `note` |

## Legacy Note

Older docs described injecting notices directly into tmux panes. That is legacy-only. Current intervention state belongs in Temporal workflows, and iTerm remains a display surface.

## Related

- [Design principles](../explanation/design-principles.md)
- [CLI reference](../reference/cli.md)
- [MCP runtime authority](../reference/mcp-tools.md)

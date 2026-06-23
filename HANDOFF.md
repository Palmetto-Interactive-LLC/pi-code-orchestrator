# Handoff: pi-code-orchestrator

## Current Direction

Lantern is being restored to a **self-contained architecture**. Each machine runs its own Lantern instance — a single Rust binary that serves as MCP server, local runner, and Temporal client. No remote server dependency.

Architecture:
- **Lantern Rust binary** serves MCP tools: `devorch_report_status`, `devorch_peer_message`, `devorch_query_team_state`, `devorch_get_setup_instructions`
- **Local Temporal instance** (127.0.0.1:8243) is the durable runtime authority for workflow state, message delivery, and recovery (Docker Temporal is strictly unsupported)
- **iTerm2** launches and displays terminal panes for agents
- **SQLite** projects local state for diagnostics and audit, not a runtime queue
- **agent-runner** executes activities locally and participates in execution windows

This is the opposite of the distributed model: no DevEnvironment `devorch-mcp-client`, no remote Temporal, no remote Postgres, no maester server.

## Repo Location

- **GitHub**: https://github.com/Palmetto-Interactive-LLC/pi-code-orchestrator
- **Extracted from**: Legacy `DevEnvironment/tools/lantern/` (distributed model)

## What's Working

- `lantern install` - installs local dependencies and helper scripts
- `lantern up/down/restart/doctor` - service lifecycle and health checks
- `lantern status` - shows local inventory and projection data from SQLite
- `lantern logs <relay|temporal>` - tails service logs
- `lantern startwork` - creates worktrees, opens an iTerm2 squad window, launches agents, and registers terminal targets
- `lantern stopwork` - closes the iTerm2 squad window, removes worktrees/branches, terminates known workflows, and marks the session stopped
- Lantern MCP server is active and serves runtime tools to agents running `lantern mcp` (see [MCP runtime authority](docs/reference/mcp-tools.md))

## Active Restoration Notes

- Restore Lantern Rust MCP tool exposure for all runtime operations (status, peer message, team-state queries, setup instructions)
- Restore local Temporal as the only runtime authority (no remote devorch-mcp-client)
- Make Lantern self-contained and deployable per machine
- Update all docs to reflect self-contained model, not distributed model

## Architecture Snapshot

Runtime authority is **local only**:

| Area | Authority |
|------|-----------|
| Session lifecycle | Local Temporal workflows |
| Setup readiness | Local Temporal workflows + agent-runner handoff |
| Runner leases | Local Temporal workflows |
| Message delivery | Local Temporal workflows → agent-runner execution window |
| Human control (pause/resume/takeover/recover/note) | Local Temporal workflows + Lantern Rust MCP |
| Diagnostics/projection | SQLite (local) + `lantern doctor` (local) |

Lantern's local Rust surfaces:

| Area | Current role |
|------|--------------|
| `startwork` | Creates worktrees, opens iTerm2 display panes, launches agents, registers local targets |
| `stopwork` | Closes iTerm2 window, cleans worktrees/branches, requests workflow cleanup |
| `terminal` | iTerm2 helper integration for launcher lifecycle and display target lookup |
| `mcp` | JSON-RPC server serving runtime tools to agents |
| `db` | Inventory, audit projection, quarantine, and doctor-state support |
| `temporal` | Worker/client integration for workflow state and control |

## Deprecated Model (2025 Distributed Architecture)

The previous architecture split runtime authority across a remote DevEnvironment node:

- DevEnvironment `devorch-mcp-client` was the MCP authority
- Workflows lived in DevEnvironment workflow package
- Lantern Rust MCP served empty tools list
- iTerm2 was display-only with no delivery authority
- Remote Postgres held workflow state

**This model is fully deprecated.** All references to `devorch-mcp-client`, remote Temporal, remote Postgres, and "Lantern MCP disabled" are legacy history only. Do not reconcile them into active operation.

## Current Documentation Entry Points

- [Docs index](docs/README.md)
- [Architecture](docs/explanation/architecture.md)
- [Temporal runtime control plane ADR](docs/architecture/adr-0001-temporal-runtime-control-plane.md)
- [MCP runtime tools](docs/reference/mcp-tools.md)
- [Doctor-state planning](docs/reference/doctor-state.md)

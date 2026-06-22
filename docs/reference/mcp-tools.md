# MCP Runtime Tools

The Lantern Rust MCP server exposes runtime tools for agents to control and query the session.

## Overview

Agents communicate with Lantern via MCP (stdin/stdout JSON-RPC 2.0). The MCP server translates tool calls into Temporal Updates, Signals, and Queries. All decisions remain in Temporal; SQLite projections are diagnostic-only.

## Available Tools

The server exposes exactly **six** tools (asserted by `tools_list_contains_all_six_tools` in `src/mcp/server.rs`). Argument names below are the authoritative schema returned by `tools/list`; `role` and `to_role` use the worker-role enum `ai | dat | sec | ops | plt | ui | doc | qa`.

#### `devorch_dispatch_task`
Dispatch a task from the Orchestrator to a worker role.

| Argument | Required | Notes |
|----------|----------|-------|
| `session` | yes | Current session ID (e.g. `navi-9`) |
| `from_role` | yes | Caller's role (normally `orchestrator`) |
| `to_role` | yes | Target worker role (enum) |
| `task_id` | yes | Task/issue identifier |
| `summary` | yes | Brief task description |
| `next_action` | no | Recommended first step |
| `files` | no | Array of relevant file paths |
| `priority` | no | `low \| normal \| high` |

Subject to the [session scope guard](#session-scope-guard).

#### `devorch_report_status`
Report the status of the caller's current task. Wired to the `orch.stateTransition` Update on the orchestrator workflow.

| Argument | Required | Notes |
|----------|----------|-------|
| `session` | yes | Current session ID |
| `role` | yes | Caller's role |
| `status` | yes | `ack \| complete \| blocked \| failed \| degraded \| recovered` |
| `task_id`, `summary`, `validation`, `next_action`, `assignment_id`, `generation`, `team_id`, `temporal_namespace`, `task_queue`, `repo_id` | no | Optional scoping/context fields |

#### `devorch_peer_message`
Send a message to, or request an action from, another worker role.

| Argument | Required | Notes |
|----------|----------|-------|
| `session` | yes | Current session ID |
| `from_role` | yes | Caller's role |
| `to_role` | yes | Target worker role (enum) |
| `info` | yes | Message body |
| `task_id`, `requested_action`, `team_id`, `temporal_namespace`, `task_queue`, `repo_id` | no | Optional context |

#### `devorch_query_team_state`
Get a snapshot of the team's current state, including active tasks and latest signals.

| Argument | Required | Notes |
|----------|----------|-------|
| `session` | yes | Current session ID |
| `team_id`, `temporal_namespace`, `task_queue`, `repo_id` | no | Optional scoping |

#### `devorch_orchestrator_inbox`
Fetch durable unacknowledged status transitions from the orchestrator inbox.

| Argument | Required | Notes |
|----------|----------|-------|
| `session` | yes | Current session ID |
| `clear_message_ids` | no | Array of message IDs to clear after reading |

Subject to the [session scope guard](#session-scope-guard).

#### `devorch_get_setup_instructions`
Get initial setup instructions and context based on the caller's role.

| Argument | Required | Notes |
|----------|----------|-------|
| `session` | yes | Current session ID |
| `role` | yes | Caller's role |
| `agent` | yes | Agent CLI (`claude`, `codex`, …) |
| `team_id`, `temporal_namespace`, `task_queue`, `repo_id` | no | Optional scoping |

## Session Scope Guard

`devorch_dispatch_task` and `devorch_orchestrator_inbox` enforce session isolation: if the agent's environment sets a non-empty `DEVORCH_SESSION`, the tool's `session` argument must match it. A mismatch returns a rejection (it is **not** a transport error — the JSON-RPC call succeeds and the content payload carries the rejection text), and the attempt is logged. This prevents one squad's agent from reading or dispatching into another squad's session. `report_status` additionally validates `team_id` / `temporal_namespace` / `repo_id` against the active session via `enforce_scope`.

## Custom Search Attributes

`lantern up` idempotently registers these eight `Keyword` search attributes on the local Temporal namespace so startwork-tagged workflows are queryable in the Temporal UI (`http://127.0.0.1:8244`):

`repo_id`, `repo_root`, `session`, `run_id`, `role`, `transport_status`, `message_status`, `delivery_status`

The first six are identity/role tags applied at workflow start; `message_status` and `delivery_status` are upserted by `MessageDeliveryWorkflow` / `SessionMessageBusWorkflow` (`packages/devorch-workflows/src/index.ts`). All eight must be registered — a workflow that upserts an unregistered attribute fails with `no mapping defined for search attribute …`.

### Human Control (CLI)

Human-initiated commands route through both Lantern CLI and Temporal:

#### `lantern pause <agent>`
Pause an agent's message delivery.

#### `lantern resume <agent>`
Resume message delivery.

#### `lantern takeover <agent>`
Human takes control of the agent pane (blocks automatic tool dispatch).

#### `lantern release <agent>`
Release human control.

#### `lantern recover <agent>`
Force recovery of a stuck agent.

#### `lantern note <agent> <message>`
Inject a note into the agent's pane and Temporal audit log.

## Protocol

- **Transport**: stdin/stdout, one JSON object per line
- **Format**: JSON-RPC 2.0
- **Methods**: `initialize`, `tools/list`, `tools/call`

## Implementation Notes

**Status: Active.** The MCP server is implemented in `src/mcp/server.rs` (request routing) and `src/mcp/tools.rs` (per-tool handlers) and serves all six tools over stdio. Agents connect by running `lantern mcp`.

All tool results are gated by Temporal workflow state. SQLite stores audit projections asynchronously and must not be treated as the source of truth for delivery, status, or control state.

### SQLite persistence

Lantern's projection state is a local SQLite database at `~/.lantern/data/relay/lantern.db`, opened via `sqlx` in WAL mode (`src/db/mod.rs`) with migrations under `migrations/`. The Temporal dev server keeps its own separate SQLite file (`~/.lantern/data/temporal/temporal.db`, via `--db-filename`). There is no PostgreSQL dependency — see [Configuration → Persistence](configuration.md#persistence).

## See also

- [Architecture](../explanation/architecture.md)
- [Temporal runtime control plane ADR](../architecture/adr-0001-temporal-runtime-control-plane.md)
- [Doctor-state planning](doctor-state.md)

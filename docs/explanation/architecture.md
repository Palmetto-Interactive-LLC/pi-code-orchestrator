# Architecture

Lantern is a self-contained agent running on each developer machine. It combines MCP server, local runtime, and Temporal client into a single Rust binary.

## System Context

```text
Agents (claude, codex, agy, kimi)
  |
  | MCP calls
  v
Lantern MCP server (stdin/stdout)
  |
  | Runtime tool calls (status, peer_message, query_team_state, etc)
  v
Lantern Rust agent
  |
  | Temporal Updates / Signals / Queries
  v
Local Temporal instance (127.0.0.1:8243)
  |
  | Workflow state, message delivery, recovery
  v
agent-runner execution windows + iTerm2 display panes
  |
  | Local process launch/output
  v
Developer workstation (git worktrees, SQLite projections)
```

Authority flows one way: Temporal workflow state decides runtime behavior. Local projections make that visible.

## Runtime Authority

All runtime decisions are owned by local Temporal workflows:

| Concern | Authority |
|---------|-----------|
| Session lifecycle | Temporal workflows (local) |
| Setup readiness | Temporal workflows + agent-runner handoff |
| Message acceptance and routing | Temporal workflows |
| Per-message delivery and acknowledgement | Temporal workflows |
| Per-role execution readiness | Temporal workflows |
| Runner heartbeat and lease state | Temporal workflows |
| MCP readiness and repair | Temporal workflows |
| Human pause/takeover/note/recover | Temporal workflows |
| Audit projection | SQLite (local) + audit workflows |

Temporal Updates are used when the caller needs an accepted/rejected result. Signals are used for notifications and control events. Queries are used for reads.

## Local Components

| Component | Role |
|-----------|------|
| `lantern` (Rust binary) | MCP server, local runner, Temporal client |
| `lantern startwork` | Creates worktrees, opens iTerm2 panes, launches agents |
| `lantern stopwork` | Closes iTerm2, cleans worktrees, terminates workflows |
| iTerm2 | Displays agent processes for humans |
| `agent-runner` | Owns local process execution and readiness handoff |
| SQLite | Stores session inventory, audit events, iTerm target projections, worktree snapshots |
| Local Temporal | Drives all runtime control plane decisions |
| `lantern doctor` | Checks local dependencies; planned doctor-state queries Temporal + SQLite |

## MCP Tools

The Lantern Rust MCP server exposes tools for agents:

- `devorch_report_status` - Report agent readiness and state
- `devorch_peer_message` - Send messages to other agents
- `devorch_query_team_state` - Query session and agent state
- `devorch_get_setup_instructions` - Retrieve setup guidance
- `pause` / `resume` / `takeover` / `release` / `recover` / `note` - Human control

All tool calls route through Temporal workflows. SQLite is updated asynchronously for audit and diagnostics.

## Startwork Flow

`lantern startwork` orchestrates session launch:

```text
launch()
  1. find_git_repo()
  2. ensure local services are available (Temporal, SQLite)
  3. create worker git worktrees
  4. build per-role environment and startup commands
  5. ensure agent MCP configuration points to local Lantern
  6. open the iTerm2 display layout
  7. launch agents through agent-runner
  8. register local session, agent, and iTerm target projections
  9. surface startup errors
```

iTerm2 is display-only. Startup prompts and runtime messages are delivered through the Temporal-gated MCP path.

## Runtime Message Flow & Advanced Design Patterns

The communication and execution fabric implements 5 state-of-the-art agentic software engineering patterns to ensure high performance and self-healing resilience:

### 1. The "Blackboard" Pattern (Shared Discovery)
* Sibling `ExecutionWindowWorkflow` instances query a centralized `BlackboardWorkflow` at the start of every task to retrieve active Discovery Cards (TypeScript compiler flag resolutions, API facts, or environment configs).
* Sibling workers automatically publish their completed, blocked, or failed outcomes as new Discovery Cards, creating a zero-overhead shared insight bulletin board.

### 2. Event-Driven Observer Protocol (MCP Event Bus)
* Agents dynamically register subscriptions to event topics (e.g. `TaskCompleted`).
* The orchestrator's MCP Event Bus broadcasts published events asynchronously to subscribed roles, injecting structured notification payloads directly into active PTY stdout streams.

### 3. "CodeAct" Execution Harness Gates (Active Validation)
* The `signal` CLI terminal harness acts as an active validation gate. When an agent attempts a `complete` transition, the harness intercepts it, runs `pnpm typecheck` or workspace builds in place, and rejects the completion locally with compiler logs if errors exist, forcing local self-correction.

### 4. Dynamic Task Re-Decomposition (Orchestrator Self-Correction)
* If an agent reports a TypeScript blocker, the `OrchestratorWorkflow` dynamically decomposes the block, spawns a high-priority troubleshooting sub-task for the `ai` role, resolves the error, and automatically resumes the parent task.

### 5. Strongly Typed JSON Schema Protocol
* Establishes gateway schema parsing and validation inside `signal` for all peer-to-peer message exchanges to eliminate conversational chatter and prompt drift.

## Orchestration Display & Remote Controls

* **Tmux Wrapping**: The orchestrator agent execution chain is wrapped in a dedicated tmux session (`devorch_orch_<session_id>`). This secures terminal scrollback history and enables remote developers to easily attach and detach (`tmux attach -t devorch_orch_<session_id>`) from anywhere, with automated cleanup routines bound to `stopwork`.

---
## Human Intervention Flow

```text
lantern pause/resume/takeover/release/recover/note
  -> Lantern Rust receives command
  -> Routes to local Temporal Signal/Update
  -> Temporal HumanControlWorkflow updates session state
  -> Delivery workflows observe updated state
  -> Local projection updated for status and audit
```

Direct pane injection is not used. iTerm panes may show the effect of a control command, but Temporal workflow state is what other runtime components read.

## Database Role

SQLite stores local support data:

- Session and agent inventory
- iTerm terminal target projections
- Audit events (sourced from Temporal via projection workflow)
- Worktree snapshots
- Doctor-state snapshots

SQLite is **not** a runtime queue or lease store. All authoritative state lives in Temporal.

## Doctor-State Direction

The current `lantern doctor` script checks local dependencies. The planned doctor-state evolution should combine:

- Dependency checks
- Temporal workflow Queries
- iTerm target readiness
- runner/MCP readiness
- Quarantine warnings
- Local projection freshness

Doctor-state is diagnostic output only; it does not make SQLite or iTerm authoritative.

## External System Boundaries

| System | Relationship |
|--------|--------------|
| Local Temporal (127.0.0.1:8243) | Durable runtime control plane — only source of truth (Docker Temporal strictly unsupported) |
| Agent CLIs (claude, codex, agy, kimi) | Downstream consumers of MCP tools |
| `agent-runner` | Local process wrapper, execution-window participant |
| iTerm2 | Display and pane-lifecycle helper |
| SQLite | Local projection, audit, and quarantine store |

## See Also

- [Design principles](design-principles.md)
- [MCP tools reference](../reference/mcp-tools.md)
- [Doctor-state planning](../reference/doctor-state.md)
- [Temporal runtime control plane ADR](../architecture/adr-0001-temporal-runtime-control-plane.md)

# Database Schema Reference

SQLite database at `~/.lantern/data/relay/lantern.db`. Migrations run automatically on first connection.

SQLite stores local inventory, audit projections, quarantine rows, and future doctor-state support. It is not the runtime source of truth for delivery, status, setup readiness, or human control.

## Current Role by Table

| Table | Role |
|-------|------|
| `machines` | Local machine inventory |
| `sessions` | Local session projection |
| `agents` | Local agent projection |
| `terminal_targets` | iTerm display target projection |
| `terminal_target_quarantine` | Legacy terminal transport quarantine |
| `events` | Local audit events |
| `transcripts` | Captured diagnostic text where available |
| `recovery_events` | Local recovery audit trail |
| `worktree_state` | Worktree inspection snapshots |
| `work_items`, `leases`, `acknowledgements` | Legacy/local migration tables only |

## Tables

### machines

| Column | Type | Description |
|--------|------|-------------|
| `id` | TEXT PK | Machine identifier |
| `created_at` | TEXT | ISO timestamp |

### sessions

| Column | Type | Description |
|--------|------|-------------|
| `id` | TEXT PK | Session ID (`{name}-{number}`) |
| `machine_id` | TEXT | Machine projection |
| `project_slug` | TEXT | Project name |
| `slot_number` | INTEGER | Slot number |
| `status` | TEXT | Local projection: `active`, `paused`, `stopping`, `stopped` |
| `created_at` | TEXT | ISO timestamp |

### agents

| Column | Type | Description |
|--------|------|-------------|
| `id` | TEXT PK | Agent ID |
| `session_id` | TEXT | Session reference |
| `role` | TEXT | Role name |
| `pane_id` | TEXT | Display pane identifier, if present |
| `worktree_path` | TEXT | Filesystem path |
| `branch` | TEXT | Git branch |
| `agent_kind` | TEXT | CLI family (`claude`, etc.) |
| `status` | TEXT | Local projection of agent availability |
| `last_seen_at` | TEXT | ISO timestamp |
| `created_at` | TEXT | ISO timestamp |

### terminal_targets

Current iTerm projection table after the iTerm migration:

| Column | Type | Description |
|--------|------|-------------|
| `agent_id` | TEXT PK | Agent reference |
| `iterm_session_id` | TEXT | iTerm2 session identifier |
| `pane_id` | TEXT | Display pane identifier, if retained |
| `transport_status` | TEXT | `ready`, `stale`, `degraded`, or `quarantined` |
| `last_seen_at` | TEXT | ISO timestamp |

This table is diagnostic/projection data only. It must not route messages by itself.

### terminal_target_quarantine

Legacy terminal rows moved aside by migration:

| Column | Type | Description |
|--------|------|-------------|
| `id` | INTEGER PK | Auto-increment |
| `agent_id` | TEXT | Agent reference |
| `legacy_tmux_session` | TEXT | Legacy tmux session value |
| `legacy_tmux_window` | TEXT | Legacy tmux window value |
| `legacy_tmux_pane` | TEXT | Legacy tmux pane value |
| `legacy_inject_method` | TEXT | Legacy injection method |
| `legacy_last_injected_at` | TEXT | Previous timestamp |
| `quarantine_reason` | TEXT | Reason the row was quarantined |
| `quarantined_at` | TEXT | ISO timestamp |

Quarantine rows are audit-only. Do not repair them into active delivery targets.

### work_items

Legacy/local migration table. Runtime message acceptance and delivery belong to Temporal workflows.

| Column | Type | Description |
|--------|------|-------------|
| `id` | TEXT PK | Work item ID |
| `session_id` | TEXT | Session reference |
| `target_role` | TEXT | Target role |
| `target_agent_id` | TEXT | Assigned agent projection |
| `task_id` | TEXT | Task identifier |
| `summary` | TEXT | Description |
| `files` | TEXT | File list projection |
| `next_action` | TEXT | Next action projection |
| `priority` | TEXT | Historical priority |
| `status` | TEXT | Historical/local status |
| `created_at` | TEXT | ISO timestamp |
| `accepted_at` | TEXT | ISO timestamp |
| `completed_at` | TEXT | ISO timestamp |

### leases

Legacy/local migration table. Runtime runner leases belong to `RunnerLeaseWorkflow`.

| Column | Type | Description |
|--------|------|-------------|
| `id` | TEXT PK | Lease ID |
| `work_item_id` | TEXT | Legacy work item reference |
| `agent_id` | TEXT | Agent reference |
| `generation` | INTEGER | Historical generation counter |
| `expires_at` | TEXT | ISO timestamp |
| `created_at` | TEXT | ISO timestamp |

### acknowledgements

Legacy/local migration table. Runtime acknowledgement belongs to `MessageDeliveryWorkflow`.

| Column | Type | Description |
|--------|------|-------------|
| `id` | TEXT PK | Ack ID |
| `work_item_id` | TEXT | Legacy work item reference |
| `agent_id` | TEXT | Agent reference |
| `ack_type` | TEXT | Acknowledgement type |
| `generation` | INTEGER | Historical generation at ack time |
| `received_at` | TEXT | ISO timestamp |

### events

| Column | Type | Description |
|--------|------|-------------|
| `id` | INTEGER PK | Auto-increment |
| `session_id` | TEXT | Session reference |
| `agent_id` | TEXT | Agent reference, nullable |
| `event_type` | TEXT | Event type |
| `payload` | TEXT | JSON payload |
| `created_at` | TEXT | ISO timestamp |

### transcripts

| Column | Type | Description |
|--------|------|-------------|
| `id` | INTEGER PK | Auto-increment |
| `agent_id` | TEXT | Agent reference |
| `content` | TEXT | Captured diagnostic content |
| `captured_at` | TEXT | ISO timestamp |

### recovery_events

| Column | Type | Description |
|--------|------|-------------|
| `id` | INTEGER PK | Auto-increment |
| `agent_id` | TEXT | Agent reference |
| `reason` | TEXT | Recovery reason |
| `old_pane_id` | TEXT | Previous display pane |
| `new_pane_id` | TEXT | New display pane |
| `generation` | INTEGER | Historical/local generation |
| `recovered_at` | TEXT | ISO timestamp |

### worktree_state

| Column | Type | Description |
|--------|------|-------------|
| `id` | INTEGER PK | Auto-increment |
| `agent_id` | TEXT | Agent reference |
| `branch` | TEXT | Branch name |
| `head_sha` | TEXT | HEAD commit |
| `dirty` | INTEGER | 0 or 1 |
| `uncommitted_files` | TEXT | File list |
| `checked_at` | TEXT | ISO timestamp |

## Inspect

```bash
sqlite3 ~/.lantern/data/relay/lantern.db ".tables"
sqlite3 ~/.lantern/data/relay/lantern.db "SELECT * FROM sessions;"
sqlite3 ~/.lantern/data/relay/lantern.db \
  "SELECT agent_id, transport_status, last_seen_at FROM terminal_targets;"
sqlite3 ~/.lantern/data/relay/lantern.db \
  "SELECT agent_id, quarantine_reason FROM terminal_target_quarantine;"
```

## See Also

- [Doctor-state planning](doctor-state.md)
- [Temporal runtime control plane ADR](../architecture/adr-0001-temporal-runtime-control-plane.md)

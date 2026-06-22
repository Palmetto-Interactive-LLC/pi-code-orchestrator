# Configuration Reference

Settings for `~/.lantern/config/lantern.toml`. Partial configs merge with defaults — empty or zero values are replaced at load time.

Configuration controls local services, paths, and projection behavior. Runtime message delivery, human control, MCP readiness, and runner leases belong to Temporal workflows, not `lantern.toml`.

## Fields

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `machine_id` | string | Hostname | Registered machine identifier |
| `lantern_dir` | path | `~/.lantern` | Root directory |
| `data_dir` | path | `~/.lantern/data` | Data storage root |
| `config_dir` | path | `~/.lantern/config` | Configuration directory |
| `logs_dir` | path | `~/.lantern/logs` | Log directory |
| `run_dir` | path | `~/.lantern/run` | PID files and sockets |
| `database_url` | string | `sqlite://~/.lantern/data/relay/lantern.db` | SQLite projection/quarantine connection URL |
| `temporal_address` | string | `127.0.0.1:8243` | Temporal gRPC endpoint (Plain `localhost` is strictly banned to prevent DNS resolution/split-brain issues with Docker) |
| `temporal_namespace` | string | `default` | Temporal namespace |
| `relay_socket_path` | path | `~/.lantern/run/relay.sock` | Relay Unix socket |
| `relay_pid_path` | path | `~/.lantern/run/relay.pid` | Relay PID file (Linux) |
| `reconciliation_interval_secs` | u64 | `5` | Local projection/check interval during migration |
| `ack_timeout_secs` | u64 | `30` | Legacy/local ack timeout; runtime delivery should use Temporal workflow state |
| `ack_retry_interval_secs` | u64 | `30` | Legacy/local retry interval; runtime delivery should use Temporal workflow state |
| `stale_threshold_secs` | u64 | `300` | Legacy/local stale threshold; runner leases belong to Temporal workflows |

## Example

```toml
machine_id = "my-macbook"
reconciliation_interval_secs = 5
ack_timeout_secs = 30
ack_retry_interval_secs = 30
stale_threshold_secs = 300
```

Template in repository: `scripts/lantern.toml`.

## Persistence

Lantern has no PostgreSQL dependency. Two independent SQLite stores back the local runtime:

| Store | Path | Owner |
|-------|------|-------|
| Projection / quarantine | `~/.lantern/data/relay/lantern.db` | Lantern (`database_url`) |
| Temporal dev server | `~/.lantern/data/temporal/temporal.db` | Temporal (`--db-filename`) |

## Temporal task queue

Relay worker polls: `lantern-{machine_id}`

Workflow state is the runtime source of truth. See [Doctor-state planning](doctor-state.md) for the planned diagnostic model that combines workflow Queries with local projection data.

## Tracing

```bash
RUST_LOG=info lantern relay
RUST_LOG=lantern=debug,sqlx=warn lantern relay
```

Service logs write to `~/.lantern/logs/` regardless of `RUST_LOG`.

## See also

- [Paths and environment](paths-and-environment.md)
- [Database schema](database-schema.md)
- [Doctor-state planning](doctor-state.md)

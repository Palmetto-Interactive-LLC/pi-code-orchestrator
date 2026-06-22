# CLI Reference

Factual reference for the `lantern` command-line interface.

**Version:** 0.1.0

## Global Options

| Option | Description |
|--------|-------------|
| `-h`, `--help` | Print help |
| `-V`, `--version` | Print version |

## Commands

### install

Install Lantern and local dependencies.

Runs `~/.lantern/bin/lantern-install` (`scripts/install.sh`).

### up

Start local services: the Temporal dev server and Relay.

### down

Stop Relay and Temporal.

### restart

Equivalent to `down` then `up`.

### doctor

Run local health checks for dependencies and services.

Current output is dependency-focused. Planned doctor-state output is documented in [Doctor-state planning](doctor-state.md).

### status

Print local projection dashboard: machine ID, sessions, agents, terminal status, work items, and recent events.

Reads from SQLite at `~/.lantern/data/relay/lantern.db`. Treat output as inventory and audit projection, not runtime authority.

### logs

```bash
lantern logs <service>
```

| Service | Aliases | Log file |
|---------|---------|----------|
| `relay` | - | `~/.lantern/logs/relay.log` |
| `temporal` | - | `~/.lantern/logs/temporal.log` |

Prints last 50 lines.

### relay

Run the Relay daemon in the foreground.

```bash
lantern relay [--machine <ID>]
```

| Flag | Default | Description |
|------|---------|-------------|
| `--machine` | `local` | Machine identifier; used for local registration and Temporal task queue naming |

Normally started by `lantern up` or launchd. Relay provides local support, compatibility MCP handshake behavior, and Temporal worker/client integration during migration.

### startwork

Launch a new squad workspace.

```bash
lantern startwork [name] [number] [--agent TYPE] [--no-init]
```

| Argument / flag | Required | Default | Description |
|-----------------|----------|---------|-------------|
| `name` | No | Repo directory name | Project slug |
| `number` | No | Next available slot | Session slot number |
| `--agent` | No | `claude` | Agent CLI: `claude`, `agy`, `codex`, `kimi` |
| `--no-init` | No | false | Skip startup prompts |

**Session ID:** `{name}-{number}`

**Creates:** iTerm2 display window with 9 panes, 8 worker git worktrees, local session/agent/terminal-target projections, and Temporal-facing runner/MCP setup state.

### stopwork

Stop a squad workspace and clean up resources.

```bash
lantern stopwork [session] [--all] [--list]
```

| Flag | Description |
|------|-------------|
| `session` | Session ID to stop, such as `myproject-1` |
| `--all` | Stop all active sessions |
| `--list` | List active sessions |

Without `session`, auto-detects from `DEVORCH_SESSION`, cwd, or sole active session.

### pause

```bash
lantern pause <agent-id>
```

Submit or record a human-control pause. Runtime authority belongs to `HumanControlWorkflow`.

### resume

```bash
lantern resume <agent-id>
```

Release a pause and allow delivery to resume once workflow state permits it.

### takeover

```bash
lantern takeover <agent-id>
```

Mark human takeover for the target agent. The command must be represented in workflow state for runtime components to honor it.

### release

```bash
lantern release <agent-id>
```

Release human takeover state.

### recover

```bash
lantern recover <agent-id>
```

Request recovery for the agent's execution window, runner, or MCP readiness.

### note

```bash
lantern note <agent-id> <message...>
```

Send a human-authored note through the human-control/runtime message path. `<message...>` accepts trailing words.

## Legacy Note

Older CLI docs described tmux sessions, direct terminal injection, and local queue scheduling as active runtime behavior. Those details are legacy-only and should not be used as current command semantics.

## Exit Codes

Non-zero on error. Service scripts propagate subprocess exit codes.

## See Also

- [Configuration](configuration.md)
- [Paths and environment](paths-and-environment.md)
- [MCP runtime authority](mcp-tools.md)

# CLAUDE.md — pi-code-orchestrator

## What This Repo Is

Lantern is a self-contained Rust binary providing local orchestration for AI coding squads. It runs on each developer machine as an MCP server, local runner (iTerm2/worktrees), and Temporal client. SQLite is the local state store.

No cloud dependency. No secrets. No credentials required.

## Build & Test

```bash
# Build
cargo build --release

# Test
cargo test

# Lint
cargo fmt --check
cargo clippy
```

Install the binary and register the launchd service:

```bash
./scripts/install.sh
source ~/.zshrc
```

Reinstall after code changes:

```bash
cargo build --release
cp target/release/lantern ~/.lantern/bin/lantern
lantern restart
```

## Secrets

None. This repo has no secrets, no cloud credentials, and no secret-management wiring.

Do not add `.op-environment`, `.envrc`, or SOPS config — this is a local CLI tool with no remote cloud or secret dependencies.

## Architecture

- Single Rust binary (`lantern`) installed to `~/.lantern/bin/`
- MCP server: serves `devorch_report_status`, `devorch_peer_message`, `devorch_query_team_state`, `devorch_get_setup_instructions`
- Local runner: manages iTerm2 terminal panes and git worktrees
- Temporal client: connects to local Temporal dev server at `127.0.0.1:8243`
- SQLite: local projection/audit state at `~/.lantern/data/relay/lantern.db` (not runtime authority)

Docker Temporal is strictly unsupported.

## Key Commands

```bash
lantern up          # Start background services (Temporal dev server + relay)
lantern down        # Stop background services
lantern doctor      # Health check all local dependencies
lantern status      # Show local inventory from SQLite
lantern startwork <project> <slot> --agent claude   # Launch a squad
lantern stopwork <project>-<slot>                   # Tear down a squad
lantern logs <relay|temporal>                       # Tail service logs
lantern mcp         # Start MCP server (for agents)
```

## CI

GitHub Actions (`.github/workflows/ci.yml`): `cargo fmt --check` on push/PR to main.

Note: `cargo check`, `clippy`, and `cargo test` are currently omitted from CI pending SQLite offline cache setup for sqlx. The ts-typecheck CI job will fail — no `package.json` exists in this repo.

## Repository

- Remote: `git@github.com-client:Palmetto-Interactive-LLC/pi-code-orchestrator.git`
- SSH alias: `github.com-client`
- Org: Palmetto-Interactive-LLC

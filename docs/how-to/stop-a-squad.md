# How to Stop a Squad

Stop a squad workspace and reclaim the iTerm window, git worktrees, branches, workflow state, and local projection records.

## Stop a Specific Session

```bash
lantern stopwork myproject-1
```

## Auto-Detect the Session

```bash
lantern stopwork
```

Detection order:

1. `DEVORCH_SESSION` environment variable
2. Current working directory inside a Lantern worktree
3. Sole active session in SQLite, if exactly one exists

If multiple sessions are active, Lantern lists them and asks you to specify one.

## List Active Sessions

```bash
lantern stopwork --list
```

## Stop All Sessions

```bash
lantern stopwork --all
```

## What Stopwork Does

1. Closes the iTerm2 window for the session.
2. Deletes git branches for worker agents.
3. Removes git worktrees with `git worktree remove --force`.
4. Prunes empty parent directories.
5. Terminates or requests cleanup for known Temporal workflows for the session.
6. Marks the local session projection `stopped` in SQLite.

Orchestrator worktree is not removed because it uses the repo root.

## If Cleanup Is Incomplete

```bash
git worktree list
git worktree prune
git branch -D myproject-ai-1
rm -rf .claude/worktrees/myproject-1
```

Then mark the local projection stopped manually if needed:

```bash
sqlite3 ~/.lantern/data/relay/lantern.db \
  "UPDATE sessions SET status='stopped' WHERE id='myproject-1';"
```

Use Temporal UI to confirm workflow cleanup:

```text
http://127.0.0.1:8244
```

## Legacy Note

Older runbooks describe killing tmux sessions during stopwork. That applied to the previous launcher only. Current cleanup closes the iTerm2 display window and relies on workflow cleanup for runtime state.

## Related

- [How to launch a squad](launch-a-squad.md)
- [CLI reference: stopwork](../reference/cli.md#stopwork)

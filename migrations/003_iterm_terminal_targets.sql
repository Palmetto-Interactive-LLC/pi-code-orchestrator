CREATE TABLE IF NOT EXISTS terminal_target_quarantine (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    agent_id TEXT NOT NULL,
    legacy_tmux_session TEXT NOT NULL,
    legacy_tmux_window TEXT NOT NULL,
    legacy_tmux_pane TEXT NOT NULL,
    legacy_inject_method TEXT NOT NULL,
    legacy_last_injected_at TEXT,
    quarantine_reason TEXT NOT NULL,
    quarantined_at TEXT NOT NULL
);

CREATE TABLE terminal_targets_new (
    agent_id TEXT PRIMARY KEY,
    iterm_session_id TEXT NOT NULL,
    pane_id TEXT,
    transport_status TEXT NOT NULL DEFAULT 'ready' CHECK (transport_status IN ('ready', 'stale', 'degraded', 'quarantined')),
    last_seen_at TEXT
);

INSERT INTO terminal_target_quarantine (
    agent_id,
    legacy_tmux_session,
    legacy_tmux_window,
    legacy_tmux_pane,
    legacy_inject_method,
    legacy_last_injected_at,
    quarantine_reason,
    quarantined_at
)
SELECT
    agent_id,
    tmux_session,
    tmux_window,
    tmux_pane,
    inject_method,
    last_injected_at,
    'legacy_terminal_transport',
    strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
FROM terminal_targets
WHERE inject_method != 'iterm_python_api' OR tmux_window != 'iterm';

INSERT INTO terminal_targets_new (
    agent_id,
    iterm_session_id,
    pane_id,
    transport_status,
    last_seen_at
)
SELECT
    agent_id,
    tmux_session,
    CASE
        WHEN inject_method = 'iterm_python_api' AND tmux_window = 'iterm' THEN tmux_pane
        ELSE NULL
    END,
    CASE
        WHEN inject_method = 'iterm_python_api' AND tmux_window = 'iterm' THEN 'ready'
        ELSE 'quarantined'
    END,
    last_injected_at
FROM terminal_targets;

DROP TABLE terminal_targets;
ALTER TABLE terminal_targets_new RENAME TO terminal_targets;

CREATE INDEX IF NOT EXISTS idx_terminal_targets_status ON terminal_targets(transport_status);

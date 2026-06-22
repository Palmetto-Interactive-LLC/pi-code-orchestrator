CREATE TABLE IF NOT EXISTS machines (
    id TEXT PRIMARY KEY,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS sessions (
    id TEXT PRIMARY KEY,
    machine_id TEXT NOT NULL,
    project_slug TEXT NOT NULL,
    slot_number INTEGER NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('active', 'paused', 'stopping', 'stopped')),
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS agents (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL,
    role TEXT NOT NULL,
    pane_id TEXT,
    worktree_path TEXT NOT NULL,
    branch TEXT NOT NULL,
    agent_kind TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('idle', 'busy', 'degraded', 'dead', 'paused', 'recovering')),
    last_seen_at TEXT,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS terminal_targets (
    agent_id TEXT PRIMARY KEY,
    tmux_session TEXT NOT NULL,
    tmux_window TEXT NOT NULL,
    tmux_pane TEXT NOT NULL,
    inject_method TEXT NOT NULL DEFAULT 'tmux_send_keys',
    last_injected_at TEXT
);

CREATE TABLE IF NOT EXISTS work_items (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL,
    target_role TEXT NOT NULL,
    target_agent_id TEXT,
    task_id TEXT NOT NULL,
    summary TEXT NOT NULL,
    files TEXT,
    next_action TEXT,
    priority TEXT NOT NULL DEFAULT 'normal',
    status TEXT NOT NULL CHECK (status IN ('pending', 'leased', 'delivered', 'acked', 'in_progress', 'blocked', 'done_claimed', 'accepted', 'stale', 'cancelled')),
    created_at TEXT NOT NULL,
    accepted_at TEXT,
    completed_at TEXT
);

CREATE TABLE IF NOT EXISTS leases (
    id TEXT PRIMARY KEY,
    work_item_id TEXT NOT NULL,
    agent_id TEXT NOT NULL,
    generation INTEGER NOT NULL DEFAULT 1,
    expires_at TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS acknowledgements (
    id TEXT PRIMARY KEY,
    work_item_id TEXT NOT NULL,
    agent_id TEXT NOT NULL,
    ack_type TEXT NOT NULL,
    generation INTEGER NOT NULL,
    received_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id TEXT NOT NULL,
    agent_id TEXT,
    event_type TEXT NOT NULL,
    payload TEXT,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS transcripts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    agent_id TEXT NOT NULL,
    content TEXT NOT NULL,
    captured_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS recovery_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    agent_id TEXT NOT NULL,
    reason TEXT NOT NULL,
    old_pane_id TEXT,
    new_pane_id TEXT,
    generation INTEGER NOT NULL,
    recovered_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS worktree_state (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    agent_id TEXT NOT NULL,
    branch TEXT NOT NULL,
    head_sha TEXT,
    dirty INTEGER NOT NULL DEFAULT 0,
    uncommitted_files TEXT,
    checked_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_agents_session ON agents(session_id);
CREATE INDEX IF NOT EXISTS idx_work_items_session ON work_items(session_id);
CREATE INDEX IF NOT EXISTS idx_work_items_status ON work_items(status);
CREATE INDEX IF NOT EXISTS idx_work_items_target ON work_items(target_role, status);
CREATE INDEX IF NOT EXISTS idx_leases_work_item ON leases(work_item_id);
CREATE INDEX IF NOT EXISTS idx_leases_agent ON leases(agent_id);
CREATE INDEX IF NOT EXISTS idx_events_session ON events(session_id);

PRAGMA foreign_keys=off;

CREATE TABLE agents_new (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL,
    role TEXT NOT NULL,
    pane_id TEXT,
    worktree_path TEXT NOT NULL,
    branch TEXT NOT NULL,
    agent_kind TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('idle', 'busy', 'degraded', 'dead', 'paused', 'recovering', 'failed')),
    last_seen_at TEXT,
    created_at TEXT NOT NULL
);

INSERT INTO agents_new (
    id,
    session_id,
    role,
    pane_id,
    worktree_path,
    branch,
    agent_kind,
    status,
    last_seen_at,
    created_at
)
SELECT
    id,
    session_id,
    role,
    pane_id,
    worktree_path,
    branch,
    agent_kind,
    status,
    last_seen_at,
    created_at
FROM agents;

DROP TABLE agents;
ALTER TABLE agents_new RENAME TO agents;

CREATE INDEX IF NOT EXISTS idx_agents_session ON agents(session_id);

PRAGMA foreign_key_check;
PRAGMA foreign_keys=on;

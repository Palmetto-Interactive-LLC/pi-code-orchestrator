-- Migration 004: tables backing the two new MCP tools.
--
-- dispatches: local projection of every devorch_dispatch_task call.
-- inbox_messages: local projection of inbound state-transition reports
--                 (populated when Temporal query is unavailable).

CREATE TABLE IF NOT EXISTS dispatches (
    message_id  TEXT PRIMARY KEY,
    session_id  TEXT NOT NULL,
    task_id     TEXT NOT NULL,
    from_role   TEXT NOT NULL,
    to_role     TEXT NOT NULL,
    summary     TEXT NOT NULL,
    files       TEXT,           -- JSON array, nullable
    next_action TEXT,
    priority    TEXT NOT NULL DEFAULT 'normal',
    created_at  TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_dispatches_session ON dispatches(session_id);
CREATE INDEX IF NOT EXISTS idx_dispatches_to_role ON dispatches(session_id, to_role);

CREATE TABLE IF NOT EXISTS inbox_messages (
    message_id  TEXT PRIMARY KEY,
    session_id  TEXT NOT NULL,
    task_id     TEXT,
    role        TEXT NOT NULL,
    status      TEXT NOT NULL,
    summary     TEXT,
    evidence    TEXT,
    next_action TEXT,
    cleared     INTEGER NOT NULL DEFAULT 0,
    received_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_inbox_session ON inbox_messages(session_id, cleared);

-- Migration 007: native-Rust delivery + auto-heal parity tables.
--
-- Replaces the former Temporal Blackboard/ExecutionWindow workflows that owned
-- discovery cards, MCP event subscriptions, and the nudge clock. SQLite is now
-- the single source of truth for all of it.

-- Blackboard discovery cards: posted on complete/blocked/failed transitions,
-- read into every dispatch prompt as shared insights. Dedupe is by card_id PK.
CREATE TABLE IF NOT EXISTS discovery_cards (
    card_id    TEXT PRIMARY KEY,
    session_id TEXT NOT NULL,
    role       TEXT NOT NULL,
    category   TEXT NOT NULL,
    summary    TEXT NOT NULL,
    solution   TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_discovery_cards_session ON discovery_cards(session_id);

-- MCP event subscriptions: a role subscribes to an event_type for a session;
-- on publish we fan out the MCP-event injection to subscribed busy panes.
CREATE TABLE IF NOT EXISTS mcp_subscriptions (
    session_id TEXT NOT NULL,
    event_type TEXT NOT NULL,
    role       TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    PRIMARY KEY (session_id, event_type, role)
);

CREATE INDEX IF NOT EXISTS idx_mcp_subscriptions_event ON mcp_subscriptions(session_id, event_type);

-- Nudge clock + active-task tracking on agents. last_signal_at advances on
-- dispatch / peer / transition (NOT heartbeat); last_nudge_at throttles nudges.
-- busy + active_task drive the nudge scan (a busy agent with an active task that
-- has gone silent gets nudged). SQLite has no ADD COLUMN IF NOT EXISTS, but the
-- forward-only runner applies each migration exactly once, so plain ADD COLUMN
-- is safe.
ALTER TABLE agents ADD COLUMN busy INTEGER NOT NULL DEFAULT 0;
ALTER TABLE agents ADD COLUMN active_task TEXT;
ALTER TABLE agents ADD COLUMN last_signal_at TEXT;
ALTER TABLE agents ADD COLUMN last_nudge_at TEXT;

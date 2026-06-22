-- The agents row is the per-(session_id, role) lease. Without a UNIQUE
-- constraint the lease is not mutually exclusive: the Rust startwork insert and
-- the TS runner's INSERT OR IGNORE each create a row (only the random PK `id` is
-- unique, so OR IGNORE never conflicts), leaking a duplicate per role.
--
-- Dedupe existing rows first — keep the row with the freshest heartbeat (the
-- live runner's row; NULLs sort last in DESC), tiebreaking on highest rowid —
-- then enforce the lease key. After this, Rust startwork uses INSERT OR IGNORE
-- and the TS runner claims the existing row via UPDATE, so exactly one row per
-- (session_id, role) survives.

DELETE FROM agents
WHERE rowid NOT IN (
    SELECT rowid FROM (
        SELECT rowid,
               ROW_NUMBER() OVER (
                   PARTITION BY session_id, role
                   ORDER BY last_heartbeat_at DESC, rowid DESC
               ) AS rn
        FROM agents
    )
    WHERE rn = 1
);

CREATE UNIQUE INDEX IF NOT EXISTS agents_session_role_uniq
    ON agents(session_id, role);

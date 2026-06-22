use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqliteSynchronous};
use sqlx::SqlitePool;
use std::str::FromStr;

#[tokio::test]
async fn terminal_target_migration_quarantines_legacy_rows() {
    let dir = tempfile::TempDir::new().unwrap();
    let db_path = dir.path().join("legacy.db");
    let database_url = format!("sqlite://{}", db_path.to_string_lossy());

    let options = SqliteConnectOptions::from_str(&database_url)
        .unwrap()
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal)
        .create_if_missing(true);
    let pool = SqlitePool::connect_with(options).await.unwrap();

    sqlx::query(
        "CREATE TABLE terminal_targets (
            agent_id TEXT PRIMARY KEY,
            tmux_session TEXT NOT NULL,
            tmux_window TEXT NOT NULL,
            tmux_pane TEXT NOT NULL,
            inject_method TEXT NOT NULL,
            last_injected_at TEXT
        )",
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO terminal_targets (agent_id, tmux_session, tmux_window, tmux_pane, inject_method, last_injected_at)
         VALUES ('agent-iterm', 'iterm-1', 'iterm', 'iterm-1', 'iterm_python_api', '2026-05-23T10:00:00Z'),
                ('agent-legacy', 'legacy-session', 'legacy-window', 'legacy-pane', 'legacy_send_keys', '2026-05-23T10:01:00Z')",
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::raw_sql(include_str!("../migrations/003_iterm_terminal_targets.sql"))
        .execute(&pool)
        .await
        .unwrap();

    let active: (String, Option<String>, String) = sqlx::query_as(
        "SELECT iterm_session_id, pane_id, transport_status FROM terminal_targets WHERE agent_id = 'agent-iterm'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(active.0, "iterm-1");
    assert_eq!(active.1.as_deref(), Some("iterm-1"));
    assert_eq!(active.2, "ready");

    let quarantined: (String,) = sqlx::query_as(
        "SELECT transport_status FROM terminal_targets WHERE agent_id = 'agent-legacy'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(quarantined.0, "quarantined");

    let quarantine_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM terminal_target_quarantine WHERE agent_id = 'agent-legacy'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(quarantine_count.0, 1);
}

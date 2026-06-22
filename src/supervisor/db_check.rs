use anyhow::Result;
use sqlx::Row;
use sqlx::SqlitePool;

use crate::db::queries::{get_agent_by_id, log_event};

async fn emit_orphan_row_event(pool: &SqlitePool, kind: &str, payload: &str) -> Result<()> {
    log_event(pool, "system", None, "stale_db_row", Some(payload)).await?;
    log_event(
        pool,
        "system",
        None,
        "temporal_recovery_required",
        Some(&format!("{}:{}", kind, payload)),
    )
    .await?;
    Ok(())
}

/// Detect orphaned or degraded local projection rows that need Temporal-led repair.
pub async fn check_stale_db_rows(pool: &SqlitePool) -> Result<()> {
    let orphaned_terminal_targets = sqlx::query(
        "SELECT tt.agent_id, tt.iterm_session_id, tt.transport_status FROM terminal_targets tt LEFT JOIN agents a ON a.id = tt.agent_id WHERE a.id IS NULL",
    )
    .fetch_all(pool)
    .await?;

    for row in orphaned_terminal_targets {
        let agent_id: String = row.get("agent_id");
        let iterm_session_id: String = row.get("iterm_session_id");
        let transport_status: String = row.get("transport_status");
        let payload = format!(
            "orphan_terminal_target agent={} iterm_session={} transport_status={}",
            agent_id, iterm_session_id, transport_status
        );
        emit_orphan_row_event(pool, "terminal_target", &payload).await?;
    }

    let terminal_targets_not_ready = sqlx::query(
        "SELECT agent_id, transport_status, iterm_session_id FROM terminal_targets WHERE transport_status != 'ready'",
    )
    .fetch_all(pool)
    .await?;

    for row in terminal_targets_not_ready {
        let agent_id: String = row.get("agent_id");
        let iterm_session_id: String = row.get("iterm_session_id");
        let status: String = row.get("transport_status");

        let agent_session = match get_agent_by_id(pool, &agent_id).await? {
            Some(agent) => agent.session_id,
            None => "system".to_string(),
        };

        let payload = format!(
            "terminal_target_not_ready agent={} session={} status={} iterm_session={}",
            agent_id, agent_session, status, iterm_session_id
        );

        log_event(
            pool,
            &agent_session,
            Some(&agent_id),
            "terminal_target_not_ready",
            Some(&payload),
        )
        .await?;

        log_event(
            pool,
            &agent_session,
            Some(&agent_id),
            "temporal_recovery_required",
            Some(&format!("terminal_target: {}", payload)),
        )
        .await?;
    }

    let orphaned_leases = sqlx::query(
        "SELECT l.id, l.work_item_id, l.agent_id FROM leases l LEFT JOIN agents a ON a.id = l.agent_id WHERE a.id IS NULL",
    )
    .fetch_all(pool)
    .await?;

    for row in orphaned_leases {
        let lease_id: String = row.get("id");
        let work_item_id: String = row.get("work_item_id");
        let agent_id: String = row.get("agent_id");
        let payload = format!(
            "orphan_lease lease_id={} agent_id={} work_item_id={}",
            lease_id, agent_id, work_item_id
        );
        emit_orphan_row_event(pool, "lease", &payload).await?;
    }

    let missing_work_items = sqlx::query(
        "SELECT l.id, l.work_item_id, l.agent_id FROM leases l LEFT JOIN work_items w ON w.id = l.work_item_id WHERE w.id IS NULL",
    )
    .fetch_all(pool)
    .await?;

    for row in missing_work_items {
        let lease_id: String = row.get("id");
        let work_item_id: String = row.get("work_item_id");
        let agent_id: String = row.get("agent_id");
        let payload = format!(
            "orphaned_work_item_ref lease_id={} agent_id={} work_item_id={}",
            lease_id, agent_id, work_item_id
        );
        emit_orphan_row_event(pool, "lease_work_item", &payload).await?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::test_helpers::create_test_pool;
    use chrono::Utc;

    #[tokio::test]
    async fn test_stale_db_rows_flags_orphan_terminal_target() {
        let (pool, _dir) = create_test_pool().await;

        sqlx::query(
            "INSERT INTO terminal_targets (agent_id, iterm_session_id, pane_id, transport_status, last_seen_at) VALUES ('agent-ghost', 'ghost-session', NULL, 'quarantined', ?)",
        )
        .bind(Utc::now().to_rfc3339())
        .execute(&pool)
        .await
        .unwrap();

        check_stale_db_rows(&pool).await.unwrap();

        let stale_events: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM events WHERE event_type = 'stale_db_row'")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert!(stale_events >= 1);

        let recovery_events: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM events WHERE event_type = 'temporal_recovery_required'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert!(recovery_events >= 1);

        let rows: Vec<(String, String)> = sqlx::query_as(
            "SELECT event_type, payload FROM events WHERE event_type IN ('stale_db_row', 'temporal_recovery_required') ORDER BY id DESC LIMIT 5",
        )
        .fetch_all(&pool)
        .await
        .unwrap();
        assert!(!rows.is_empty());
    }

    #[tokio::test]
    async fn test_stale_db_rows_warn_terminal_target_projection() {
        let (pool, _dir) = create_test_pool().await;

        sqlx::query(
            "INSERT INTO terminal_targets (agent_id, iterm_session_id, pane_id, transport_status, last_seen_at) VALUES ('agent-ghost', 'ghost-session', NULL, 'degraded', ?)",
        )
        .bind(Utc::now().to_rfc3339())
        .execute(&pool)
        .await
        .unwrap();

        check_stale_db_rows(&pool).await.unwrap();

        let target_events: Vec<String> = sqlx::query_scalar(
            "SELECT payload FROM events WHERE event_type = 'terminal_target_not_ready'",
        )
        .fetch_all(&pool)
        .await
        .unwrap();
        assert!(target_events
            .first()
            .expect("event should exist")
            .contains("agent-ghost"));
    }
}

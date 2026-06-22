use std::sync::OnceLock;
use std::time::Duration;

use anyhow::Result;
use sqlx::SqlitePool;
use tokio::time::interval;
use tracing::{error, info};

mod db_check;
mod lease_check;
mod process_check;
mod worktree_check;

static DB_POOL: OnceLock<SqlitePool> = OnceLock::new();

/// Initialize the supervisor with a database pool.
pub fn init(pool: SqlitePool) {
    let _ = DB_POOL.set(pool);
}

fn get_pool() -> Result<SqlitePool> {
    DB_POOL
        .get()
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("supervisor db pool not initialized"))
}

/// Spawn the continuous reconciliation loop.
pub fn start_reconciliation_loop(period: Duration) {
    tokio::spawn(async move {
        let mut ticker = interval(period);
        loop {
            ticker.tick().await;
            if let Err(e) = reconcile_once().await {
                error!(error = %e, "reconciliation iteration failed");
            }
        }
    });
}

/// Run a single reconciliation pass.
pub async fn reconcile_once() -> Result<()> {
    let pool = get_pool()?;

    info!("starting reconciliation iteration");

    // 1. List all active sessions from SQLite
    let session_ids: Vec<String> =
        sqlx::query_scalar("SELECT id FROM sessions WHERE status = 'active'")
            .fetch_all(&pool)
            .await?;

    info!(session_count = session_ids.len(), "loaded active sessions");

    for session_id in &session_ids {
        // 2. For each session, list expected agents
        let agents = match crate::db::queries::get_agents_for_session(&pool, session_id).await {
            Ok(a) => a,
            Err(e) => {
                error!(session_id = %session_id, error = %e, "failed to get agents for session");
                continue;
            }
        };

        info!(session_id = %session_id, agent_count = agents.len(), "reconciling session");

        // 3. Check process health
        if let Err(e) = process_check::check_agent_processes(&pool, session_id, &agents).await {
            error!(session_id = %session_id, error = %e, "agent process check failed");
        }

        // 4. Check git worktrees
        if let Err(e) = worktree_check::check_worktrees(&pool, session_id, &agents).await {
            error!(session_id = %session_id, error = %e, "worktree check failed");
        }
    }

    // Global checks
    if let Err(e) = worktree_check::detect_dirty_worktrees(&pool).await {
        error!(error = %e, "dirty worktree detection failed");
    }
    if let Err(e) = lease_check::check_stale_leases(&pool).await {
        error!(error = %e, "stale lease check failed");
    }
    if let Err(e) = db_check::check_stale_db_rows(&pool).await {
        error!(error = %e, "stale db row check failed");
    }
    if let Err(e) = process_check::check_relay_health(&pool).await {
        error!(error = %e, "relay health check failed");
    }

    info!("reconciliation iteration complete");
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn supervisor_checks_remain_audit_only_without_terminal_injection_calls() {
        let sources = [
            include_str!("lease_check.rs"),
            include_str!("process_check.rs"),
            include_str!("worktree_check.rs"),
            include_str!("db_check.rs"),
        ];

        for source in sources {
            assert!(
                !source.contains("crate::terminal::"),
                "supervisor checks must not own terminal write path"
            );
            assert!(
                !source.contains("inject_message("),
                "supervisor checks must not write to terminal targets"
            );
            assert!(
                !source.contains("active_md::write_active_md"),
                "supervisor checks must not inject recovery artifacts"
            );
        }
    }
}

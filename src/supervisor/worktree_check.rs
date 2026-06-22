use anyhow::Result;
use sqlx::{Row, SqlitePool};
use tracing::{info, warn};

use crate::db::queries::log_event;
use crate::types::Agent;

/// Verify worktree paths exist and branches match SQLite expectations.
pub async fn check_worktrees(pool: &SqlitePool, session_id: &str, agents: &[Agent]) -> Result<()> {
    for agent in agents {
        let path = std::path::Path::new(&agent.worktree_path);

        if !path.exists() {
            warn!(
                session_id = %session_id,
                agent_id = %agent.id,
                path = %agent.worktree_path,
                "worktree path missing"
            );
            log_event(
                pool,
                session_id,
                Some(&agent.id),
                "worktree_missing",
                Some(&agent.worktree_path),
            )
            .await?;
            log_event(
                pool,
                session_id,
                Some(&agent.id),
                "temporal_recovery_required",
                Some("missing_worktree"),
            )
            .await?;
            continue;
        }

        let output = tokio::process::Command::new("git")
            .arg("-C")
            .arg(&agent.worktree_path)
            .arg("rev-parse")
            .arg("--abbrev-ref")
            .arg("HEAD")
            .output()
            .await?;

        if output.status.success() {
            let current_branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if current_branch != agent.branch {
                warn!(
                    session_id = %session_id,
                    agent_id = %agent.id,
                    expected = %agent.branch,
                    actual = %current_branch,
                    "branch mismatch"
                );
                log_event(
                    pool,
                    session_id,
                    Some(&agent.id),
                    "branch_mismatch",
                    Some(&current_branch),
                )
                .await?;
            }
        } else {
            warn!(
                session_id = %session_id,
                agent_id = %agent.id,
                path = %agent.worktree_path,
                "failed to read current branch"
            );
        }
    }

    Ok(())
}

/// Detect worktrees with uncommitted changes.
pub async fn detect_dirty_worktrees(pool: &SqlitePool) -> Result<()> {
    let rows =
        sqlx::query("SELECT id, session_id, worktree_path FROM agents WHERE status != 'dead'")
            .fetch_all(pool)
            .await?;

    for row in rows {
        let agent_id: String = row.get("id");
        let session_id: String = row.get("session_id");
        let worktree_path: String = row.get("worktree_path");

        let output = tokio::process::Command::new("git")
            .arg("-C")
            .arg(&worktree_path)
            .arg("status")
            .arg("--porcelain")
            .output()
            .await?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if !stdout.trim().is_empty() {
                info!(
                    agent_id = %agent_id,
                    path = %worktree_path,
                    "dirty worktree detected"
                );
                log_event(
                    pool,
                    &session_id,
                    Some(&agent_id),
                    "dirty_worktree",
                    Some(&worktree_path),
                )
                .await?;
            }
        } else {
            warn!(
                agent_id = %agent_id,
                path = %worktree_path,
                "failed to check worktree dirtiness"
            );
        }
    }

    Ok(())
}

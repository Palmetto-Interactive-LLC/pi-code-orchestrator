use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tokio::process::Command;
use tracing::{info, warn};

use crate::db::queries::{get_agent_by_id, log_event};
use crate::types::Assignment;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliverResult {
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub clean: bool,
    pub branch: Option<String>,
    pub dirty_files: Vec<String>,
    pub ahead: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptResult {
    pub lines: Vec<String>,
    pub byte_count: usize,
}

/// Record assignment delivery as unsupported by the Rust local runtime.
pub async fn deliver_assignment(
    pool: &SqlitePool,
    assignment: Assignment,
) -> Result<DeliverResult> {
    log_event(
        pool,
        &assignment.session_id,
        None,
        "assignment_temporal_required",
        Some(&serde_json::to_string(&assignment)?),
    )
    .await?;

    info!(assignment_id = %assignment.assignment_id, "Assignment requires Temporal workflow delivery");

    Ok(DeliverResult {
        status: "temporal_workflow_required".to_string(),
    })
}

/// Check git status in the agent's worktree.
pub async fn validate_worktree(pool: &SqlitePool, agent_id: &str) -> Result<ValidationResult> {
    let agent = match get_agent_by_id(pool, agent_id).await? {
        Some(a) => a,
        None => {
            warn!(%agent_id, "Agent not found for validation");
            return Ok(ValidationResult {
                clean: false,
                branch: None,
                dirty_files: vec![],
                ahead: false,
            });
        }
    };

    let output = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(&agent.worktree_path)
        .output()
        .await?;

    let status_str = String::from_utf8_lossy(&output.stdout);
    let dirty_files: Vec<String> = status_str
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.to_string())
        .collect();

    let branch_output = Command::new("git")
        .args(["branch", "--show-current"])
        .current_dir(&agent.worktree_path)
        .output()
        .await?;

    let branch = String::from_utf8_lossy(&branch_output.stdout)
        .trim()
        .to_string();
    let branch = if branch.is_empty() {
        None
    } else {
        Some(branch)
    };

    let ahead = dirty_files
        .iter()
        .any(|l| l.starts_with('M') || l.starts_with('A') || l.starts_with('D'));

    Ok(ValidationResult {
        clean: dirty_files.is_empty(),
        branch,
        dirty_files,
        ahead,
    })
}

/// Transcript capture is owned by runner/workflow activity integration.
pub async fn capture_transcript(pool: &SqlitePool, agent_id: &str) -> Result<TranscriptResult> {
    log_event(
        pool,
        "system",
        Some(agent_id),
        "transcript_temporal_required",
        None,
    )
    .await?;
    Ok(TranscriptResult {
        lines: vec![],
        byte_count: 0,
    })
}

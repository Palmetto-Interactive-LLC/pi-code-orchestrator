//! Native auto-heal: the 30s nudge loop.
//!
//! Replaces the former Temporal ExecutionWindow nudge timer. Every 30s we scan
//! agents that are busy with an active task and, if one has gone silent past the
//! threshold (and we haven't nudged it too recently), we inject the byte-exact
//! `[ORCHESTRATOR NUDGE]` prompt into its pane, stamp last_nudge_at, and record a
//! `degraded` transition in the inbox.
//!
//! `last_signal_at` is advanced on dispatch / peer / transition (see
//! `db::queries::mark_agent_signal`) — NOT on heartbeat — so the silence clock
//! measures genuine lack of forward progress, matching the TS semantics.

use chrono::Utc;
use sqlx::SqlitePool;
use tokio::sync::watch;
use tracing::{info, warn};

use crate::db::queries;
use crate::delivery::inject;

/// Silence threshold before a busy agent is nudged (15 min).
const NUDGE_SILENCE_MS: i64 = 900_000;
/// Minimum gap between consecutive nudges to the same agent (90s).
const NUDGE_THROTTLE_MS: i64 = 90_000;
/// How often the scan runs.
const NUDGE_SCAN_SECS: u64 = 30;

/// Byte-exact nudge prompt (parity with the former `formatNudgePrompt`).
fn format_nudge_prompt(role: &str, task_id: &str, duration_min: i64) -> String {
    let mut lines: Vec<String> = Vec::new();
    lines.push("\n\n>>> [ORCHESTRATOR NUDGE] <<<".to_string());
    lines.push(format!(
        "Attention {role}: You have been working on task \"{task_id}\" for {duration_min} minutes without emitting a status transition."
    ));
    lines.push(
        "The team is waiting for your progress. Please respond immediately by running one of the following:"
            .to_string(),
    );
    lines.push(format!(
        "  - If you are still working:   signal --status progress --task {task_id} --summary \"updating progress...\""
    ));
    lines.push(format!(
        "  - If you are completed:       signal --status complete --task {task_id} --summary \"description...\" --evidence \"...\""
    ));
    lines.push(format!(
        "  - If you are blocked:         signal --status blocked --task {task_id} --summary \"description...\" --blocker \"...\""
    ));
    lines.push(format!(
        "  - If you are failed:          signal --status failed --task {task_id} --summary \"description...\""
    ));
    lines.push("\n".to_string());
    lines.join("\n")
}

/// Spawn the 30s nudge loop. Follows the existing relay tokio::spawn + watch
/// shutdown pattern in `main.rs`.
pub fn start_nudge_loop(pool: SqlitePool, mut shutdown: watch::Receiver<bool>) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(NUDGE_SCAN_SECS));
        loop {
            tokio::select! {
                _ = interval.tick() => {
                    if let Err(e) = nudge_once(&pool).await {
                        warn!(error = %e, "nudge scan failed");
                    }
                }
                _ = shutdown.changed() => {
                    if *shutdown.borrow() {
                        info!("Nudge loop shutting down");
                        break;
                    }
                }
            }
        }
    });
}

/// One nudge pass. Public for tests.
pub async fn nudge_once(pool: &SqlitePool) -> anyhow::Result<()> {
    let now = Utc::now();
    let candidates = queries::get_nudge_candidates(pool).await?;

    for cand in candidates {
        let silence_ms = cand
            .last_signal_at
            .map(|t| (now - t).num_milliseconds())
            .unwrap_or(i64::MAX);
        if silence_ms <= NUDGE_SILENCE_MS {
            continue;
        }
        let since_nudge_ms = cand
            .last_nudge_at
            .map(|t| (now - t).num_milliseconds())
            .unwrap_or(i64::MAX);
        if since_nudge_ms <= NUDGE_THROTTLE_MS {
            continue;
        }

        let duration_min = silence_ms / 60_000;
        let prompt = format_nudge_prompt(&cand.role, &cand.active_task, duration_min);

        if let Err(e) = inject::deliver_to_role(pool, &cand.session_id, &cand.role, &prompt).await {
            warn!(role = %cand.role, error = %e, "nudge injection failed (degraded pane)");
        }

        // Throttle clock + degraded transition record (parity with the TS
        // window emitting a `degraded` state transition on nudge).
        let _ = queries::mark_agent_nudged(pool, &cand.agent_id).await;
        let summary = format!("orchestrator nudged silent agent (silent for {duration_min}m)");
        let _ = sqlx::query(
            "INSERT INTO inbox_messages \
             (message_id, session_id, task_id, role, status, summary, received_at) \
             VALUES (?, ?, ?, ?, 'degraded', ?, ?)",
        )
        .bind(crate::types::generate_id("nudge"))
        .bind(&cand.session_id)
        .bind(&cand.active_task)
        .bind(&cand.role)
        .bind(&summary)
        .bind(now.to_rfc3339())
        .execute(pool)
        .await;

        info!(role = %cand.role, task = %cand.active_task, duration_min, "nudged silent busy agent");
    }
    Ok(())
}

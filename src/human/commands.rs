use anyhow::{anyhow, Result};
use serde::Serialize;
use tracing::info;
use uuid::Uuid;

use crate::db::queries;
use crate::temporal::signals;

use super::pool;

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
enum HumanControlAction {
    Pause,
    Resume,
    Takeover,
    Release,
    Note,
    Recover,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SessionIdentity {
    repo_id: String,
    repo_root: String,
    session: String,
    run_id: String,
    role: Option<String>,
    temporal_namespace: String,
    task_queue: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct HumanControlCommand {
    command_id: String,
    identity: SessionIdentity,
    action: HumanControlAction,
    target_role: String,
    requested_by: Option<String>,
    note: Option<String>,
    requested_at: String,
}

fn required_env(name: &str) -> Result<String> {
    std::env::var(name).map_err(|_| anyhow!("{} is required for Temporal human control", name))
}

fn identity_from_env() -> Result<SessionIdentity> {
    Ok(SessionIdentity {
        repo_id: required_env("DEVORCH_REPO_ID")?,
        repo_root: required_env("DEVORCH_REPO_ROOT")?,
        session: required_env("DEVORCH_SESSION")?,
        run_id: required_env("DEVORCH_RUN_ID")?,
        role: std::env::var("DEVORCH_ROLE").ok(),
        temporal_namespace: required_env("DEVORCH_TEMPORAL_NAMESPACE")?,
        task_queue: required_env("DEVORCH_TASK_QUEUE")?,
    })
}

async fn submit_human_control(
    agent_id: &str,
    action: HumanControlAction,
    note: Option<&str>,
) -> Result<()> {
    let pool = pool()?;
    let identity = identity_from_env()?;

    // Look up agent first by exact ID. If not found, try to resolve as a role in the current session.
    let agent = match queries::get_agent_by_id(pool, agent_id).await? {
        Some(agent) => agent,
        None => {
            let session_agents = queries::get_agents_for_session(pool, &identity.session).await?;
            let normalized_role = match agent_id {
                "orch" | "orchestrator" => "orchestrator",
                other => other,
            };
            session_agents
                .into_iter()
                .find(|a| a.role == normalized_role)
                .ok_or_else(|| {
                    anyhow!(
                        "Agent or Role {} not found in session {}",
                        agent_id,
                        identity.session
                    )
                })?
        }
    };

    if identity.session != agent.session_id {
        anyhow::bail!(
            "DEVORCH_SESSION {} does not match agent {} session {}",
            identity.session,
            agent_id,
            agent.session_id
        );
    }

    let command = HumanControlCommand {
        command_id: Uuid::new_v4().to_string(),
        identity,
        action,
        target_role: agent.role.clone(),
        requested_by: std::env::var("USER").ok(),
        note: note.map(ToOwned::to_owned),
        requested_at: chrono::Utc::now().to_rfc3339(),
    };
    let workflow_id = signals::workflow_id_for_human_control(
        &command.identity.repo_id,
        &command.identity.session,
        &command.identity.run_id,
    );
    let payload = serde_json::to_value(&command)?;
    queries::log_event(
        pool,
        &agent.session_id,
        Some(&agent.id),
        "human_control_requested",
        Some(&payload.to_string()),
    )
    .await?;

    let config = crate::config::Config::load()?;
    signals::signal_human_control_command(
        &config.temporal_address,
        &command.identity.temporal_namespace,
        &workflow_id,
        payload,
    )
    .await?;

    info!(
        agent_id = %agent.id,
        workflow_id, "Human control command submitted to Temporal"
    );
    Ok(())
}

/// Request that Temporal pause an agent.
pub async fn pause_agent(agent_id: &str) -> Result<()> {
    submit_human_control(agent_id, HumanControlAction::Pause, None).await
}

/// Request that Temporal resume normal operation for an agent.
pub async fn resume_agent(agent_id: &str) -> Result<()> {
    submit_human_control(agent_id, HumanControlAction::Resume, None).await
}

/// Request explicit human control of an agent.
pub async fn takeover_agent(agent_id: &str) -> Result<()> {
    submit_human_control(agent_id, HumanControlAction::Takeover, None).await
}

/// Request release of human control for an agent.
pub async fn release_agent(agent_id: &str) -> Result<()> {
    submit_human_control(agent_id, HumanControlAction::Release, None).await
}

/// Request recovery for an agent.
pub async fn recover_agent(agent_id: &str) -> Result<()> {
    submit_human_control(agent_id, HumanControlAction::Recover, None).await
}

/// Send a human-authored note through Temporal-owned control.
pub async fn note_agent(agent_id: &str, message: &str) -> Result<()> {
    submit_human_control(agent_id, HumanControlAction::Note, Some(message)).await
}

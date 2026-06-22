use anyhow::Result;
use serde::de::DeserializeOwned;
use sqlx::SqlitePool;
use tokio::time::{sleep, Duration};
use tracing::{error, info, warn};

use crate::temporal::activities;
use crate::temporal::client::{
    Payload, Payloads, PollActivityTaskQueueRequest, RespondActivityTaskCompletedRequest,
    TaskQueue, TemporalGrpcClient,
};
use crate::types::generate_id;

pub struct Worker {
    client: TemporalGrpcClient,
    db_pool: SqlitePool,
    task_queue: String,
    namespace: String,
    identity: String,
}

impl Worker {
    pub async fn new(db_pool: SqlitePool, addr: &str, task_queue: &str) -> Result<Self> {
        let client = TemporalGrpcClient::connect(addr.to_string()).await?;
        Ok(Self {
            client,
            db_pool,
            task_queue: task_queue.to_string(),
            namespace: "default".to_string(),
            identity: format!("lantern-worker-{}", generate_id("")),
        })
    }

    pub async fn run(&mut self, mut shutdown: tokio::sync::watch::Receiver<bool>) -> Result<()> {
        info!(
            identity = %self.identity,
            queue = %self.task_queue,
            "Temporal worker started"
        );

        while !*shutdown.borrow_and_update() {
            if shutdown.has_changed().unwrap_or(false) && *shutdown.borrow() {
                break;
            }

            match self.poll_and_handle().await {
                Ok(true) => {}
                Ok(false) => {
                    sleep(Duration::from_millis(500)).await;
                }
                Err(e) => {
                    error!("Worker poll/handle error: {:?}", e);
                    sleep(Duration::from_secs(2)).await;
                }
            }
        }

        info!("Temporal worker shutting down");
        Ok(())
    }

    async fn poll_and_handle(&mut self) -> Result<bool> {
        let request = PollActivityTaskQueueRequest {
            namespace: self.namespace.clone(),
            task_queue: TaskQueue {
                name: self.task_queue.clone(),
                kind: 1, // TASK_QUEUE_KIND_NORMAL
            },
            identity: self.identity.clone(),
            worker_version_capabilities: None,
        };

        let response = self.client.poll_activity_task_queue(request).await?;

        let task_token = match response.task_token {
            Some(t) => t,
            None => return Ok(false),
        };

        let activity_type = response
            .activity_type
            .map(|t| t.name)
            .unwrap_or_else(|| "unknown".to_string());

        info!(%activity_type, "Received activity task");

        let handle_result = match activity_type.as_str() {
            "deliver_assignment" => {
                let input = self.parse_payload::<crate::types::Assignment>(&response.input)?;
                let result = activities::deliver_assignment(&self.db_pool, input).await?;
                self.complete_task(task_token, result).await
            }
            "validate_worktree" => {
                let agent_id = self.parse_payload::<String>(&response.input)?;
                let result = activities::validate_worktree(&self.db_pool, &agent_id).await?;
                self.complete_task(task_token, result).await
            }
            "capture_transcript" => {
                let agent_id = self.parse_payload::<String>(&response.input)?;
                let result = activities::capture_transcript(&self.db_pool, &agent_id).await?;
                self.complete_task(task_token, result).await
            }
            other => {
                warn!(%other, "Unknown activity type, failing task");
                self.fail_task(task_token, &format!("Unknown activity type: {}", other))
                    .await
            }
        };

        if let Err(e) = handle_result {
            error!(%activity_type, "Failed to complete activity: {:?}", e);
        }

        Ok(true)
    }

    fn parse_payload<T: DeserializeOwned>(&self, payloads: &Option<Payloads>) -> Result<T> {
        let payloads = payloads
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No input payloads"))?;
        let payload = payloads
            .payloads
            .first()
            .ok_or_else(|| anyhow::anyhow!("Empty payloads"))?;
        let data = payload.data.as_bytes();
        Ok(serde_json::from_slice(data)?)
    }

    async fn complete_task(
        &mut self,
        task_token: String,
        result: impl serde::Serialize,
    ) -> Result<()> {
        let json = serde_json::to_vec(&result)?;
        let request = RespondActivityTaskCompletedRequest {
            task_token,
            result: Some(Payloads {
                payloads: vec![Payload {
                    metadata: [("encoding".to_string(), "json/plain".to_string())].into(),
                    data: String::from_utf8_lossy(&json).to_string(),
                }],
            }),
            identity: self.identity.clone(),
        };

        self.client.respond_activity_task_completed(request).await?;
        info!("Activity task completed successfully");
        Ok(())
    }

    async fn fail_task(&mut self, task_token: String, message: &str) -> Result<()> {
        // Simplified: return the error as a JSON payload inside a "completed" response.
        // A production worker should use RespondActivityTaskFailed.
        let request = RespondActivityTaskCompletedRequest {
            task_token,
            result: Some(Payloads {
                payloads: vec![Payload {
                    metadata: [("encoding".to_string(), "json/plain".to_string())].into(),
                    data: serde_json::json!({ "error": message }).to_string(),
                }],
            }),
            identity: self.identity.clone(),
        };

        self.client.respond_activity_task_completed(request).await?;
        warn!("Activity task marked as failed: {}", message);
        Ok(())
    }
}

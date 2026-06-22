use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::process::Command;
use tonic::codec::ProstCodec;

// Proto types used for correct protobuf wire encoding on the gRPC transport.
use temporal_sdk_core_protos::temporal::api::{
    common::v1 as proto_common, taskqueue::v1 as proto_taskqueue, workflowservice::v1 as proto_ws,
};

// ---------------------------------------------------------------------------
// Simplified Temporal message types (JSON-over-gRPC)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TaskQueue {
    pub name: String,
    pub kind: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkerVersionCapabilities {
    pub build_id: String,
    pub use_versioning: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ActivityType {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkflowExecution {
    pub workflow_id: String,
    pub run_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Payload {
    pub metadata: std::collections::HashMap<String, String>,
    pub data: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Payloads {
    pub payloads: Vec<Payload>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PollActivityTaskQueueRequest {
    pub namespace: String,
    pub task_queue: TaskQueue,
    pub identity: String,
    pub worker_version_capabilities: Option<WorkerVersionCapabilities>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PollActivityTaskQueueResponse {
    pub task_token: Option<String>,
    pub activity_id: Option<String>,
    pub activity_type: Option<ActivityType>,
    pub input: Option<Payloads>,
    pub workflow_execution: Option<WorkflowExecution>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RespondActivityTaskCompletedRequest {
    pub task_token: String,
    pub result: Option<Payloads>,
    pub identity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RespondActivityTaskCompletedResponse {}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SignalWorkflowExecutionRequest {
    pub namespace: String,
    pub workflow_execution: WorkflowExecution,
    pub signal_name: String,
    pub input: Option<Payloads>,
    pub identity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SignalWorkflowExecutionResponse {}

// ---------------------------------------------------------------------------
// Temporal gRPC client
// ---------------------------------------------------------------------------

pub struct TemporalGrpcClient {
    inner: tonic::client::Grpc<tonic::transport::Channel>,
    cli_address: String,
}

fn normalize_temporal_address(addr: &str) -> String {
    addr.trim_start_matches("http://")
        .trim_start_matches("https://")
        .to_string()
}

fn payload_to_metadata_args(payload: &Payload) -> Vec<String> {
    // `temporal ... --input-meta key=value` stores `value` as the raw metadata
    // bytes. The value must be the literal string (e.g. `encoding=json/plain`) —
    // JSON-stringifying it produces `encoding="json/plain"` (with quotes), which
    // the TypeScript SDK's payload converter rejects with `Unknown encoding`.
    payload
        .metadata
        .iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect()
}

fn map_temporal_error(message: &str) -> tonic::Status {
    let lower = message.to_lowercase();
    if lower.contains("already exists")
        || lower.contains("already started")
        || lower.contains("already running")
    {
        return tonic::Status::already_exists("workflow already exists");
    }

    if lower.contains("not found") {
        return tonic::Status::not_found("workflow not found");
    }

    tonic::Status::internal(message)
}

impl TemporalGrpcClient {
    pub async fn connect(addr: String) -> Result<Self> {
        let addr = if addr.starts_with("http://") || addr.starts_with("https://") {
            addr
        } else {
            format!("http://{addr}")
        };
        let cli_address = normalize_temporal_address(&addr);

        let channel = tonic::transport::Channel::from_shared(addr)?
            .connect()
            .await?;
        Ok(Self {
            inner: tonic::client::Grpc::new(channel),
            cli_address,
        })
    }

    pub async fn poll_activity_task_queue(
        &mut self,
        request: PollActivityTaskQueueRequest,
    ) -> std::result::Result<PollActivityTaskQueueResponse, tonic::Status> {
        self.inner
            .ready()
            .await
            .map_err(|e| tonic::Status::unavailable(format!("transport: {e}")))?;

        // Build the real protobuf request so the server can parse the wire format.
        #[allow(deprecated)]
        let proto_req = proto_ws::PollActivityTaskQueueRequest {
            namespace: request.namespace,
            task_queue: Some(proto_taskqueue::TaskQueue {
                name: request.task_queue.name,
                kind: request.task_queue.kind,
                normal_name: String::new(),
            }),
            identity: request.identity,
            task_queue_metadata: None,
            worker_version_capabilities: None,
            deployment_options: None,
            worker_heartbeat: None,
        };

        let codec = ProstCodec::<
            proto_ws::PollActivityTaskQueueRequest,
            proto_ws::PollActivityTaskQueueResponse,
        >::default();
        let path: http::uri::PathAndQuery =
            "/temporal.api.workflowservice.v1.WorkflowService/PollActivityTaskQueue"
                .parse()
                .map_err(|e| tonic::Status::internal(format!("path parse: {e}")))?;

        // Temporal's PollActivityTaskQueue requires a gRPC deadline; 70 s gives the
        // server its standard 60 s long-poll window plus margin.
        let mut tonic_req = tonic::Request::new(proto_req);
        tonic_req.set_timeout(Duration::from_secs(70));

        let proto_resp = self.inner.unary(tonic_req, path, codec).await?.into_inner();

        // An empty task_token means no task was available (poll timed out).
        if proto_resp.task_token.is_empty() {
            return Ok(PollActivityTaskQueueResponse::default());
        }

        // Encode task_token as hex so it round-trips through the String-typed facade.
        let task_token = hex::encode(&proto_resp.task_token);

        let activity_type = proto_resp
            .activity_type
            .map(|t| ActivityType { name: t.name });

        let workflow_execution = proto_resp.workflow_execution.map(|we| WorkflowExecution {
            workflow_id: we.workflow_id,
            run_id: we.run_id,
        });

        let input = proto_resp.input.map(|payloads| Payloads {
            payloads: payloads
                .payloads
                .into_iter()
                .map(|p| Payload {
                    metadata: p
                        .metadata
                        .into_iter()
                        .map(|(k, v)| (k, String::from_utf8_lossy(&v).into_owned()))
                        .collect(),
                    data: String::from_utf8_lossy(&p.data).into_owned(),
                })
                .collect(),
        });

        Ok(PollActivityTaskQueueResponse {
            task_token: Some(task_token),
            activity_id: Some(proto_resp.activity_id),
            activity_type,
            input,
            workflow_execution,
        })
    }

    pub async fn respond_activity_task_completed(
        &mut self,
        request: RespondActivityTaskCompletedRequest,
    ) -> std::result::Result<RespondActivityTaskCompletedResponse, tonic::Status> {
        self.inner
            .ready()
            .await
            .map_err(|e| tonic::Status::unavailable(format!("transport: {e}")))?;

        // Decode the hex-encoded task_token back to raw bytes.
        let task_token = hex::decode(&request.task_token)
            .map_err(|e| tonic::Status::invalid_argument(format!("invalid task_token: {e}")))?;

        let result = request.result.map(|payloads| proto_common::Payloads {
            payloads: payloads
                .payloads
                .into_iter()
                .map(|p| proto_common::Payload {
                    metadata: p
                        .metadata
                        .into_iter()
                        .map(|(k, v)| (k, v.into_bytes()))
                        .collect(),
                    data: p.data.into_bytes(),
                })
                .collect(),
        });

        #[allow(deprecated)]
        let proto_req = proto_ws::RespondActivityTaskCompletedRequest {
            task_token,
            result,
            identity: request.identity,
            namespace: String::new(),
            worker_version: None,
            deployment: None,
            deployment_options: None,
        };

        let codec = ProstCodec::<
            proto_ws::RespondActivityTaskCompletedRequest,
            proto_ws::RespondActivityTaskCompletedResponse,
        >::default();
        let path: http::uri::PathAndQuery =
            "/temporal.api.workflowservice.v1.WorkflowService/RespondActivityTaskCompleted"
                .parse()
                .map_err(|e| tonic::Status::internal(format!("path parse: {e}")))?;

        self.inner
            .unary(tonic::Request::new(proto_req), path, codec)
            .await?;
        Ok(RespondActivityTaskCompletedResponse {})
    }

    pub async fn signal_workflow_execution(
        &mut self,
        request: SignalWorkflowExecutionRequest,
    ) -> std::result::Result<SignalWorkflowExecutionResponse, tonic::Status> {
        let mut args = vec![
            "workflow".to_string(),
            "signal".to_string(),
            "--workflow-id".to_string(),
            request.workflow_execution.workflow_id,
            "--name".to_string(),
            request.signal_name,
            "--namespace".to_string(),
            request.namespace,
            "--identity".to_string(),
            request.identity,
            "--output".to_string(),
            "json".to_string(),
            "--address".to_string(),
            self.cli_address.clone(),
        ];

        if !request.workflow_execution.run_id.is_empty() {
            args.push("--run-id".to_string());
            args.push(request.workflow_execution.run_id);
        }

        if let Some(payloads) = request.input {
            for payload in payloads.payloads {
                for meta in payload_to_metadata_args(&payload) {
                    args.push("--input-meta".to_string());
                    args.push(meta);
                }
                args.push("--input".to_string());
                args.push(payload.data);
            }
        }

        let output = Command::new("temporal")
            .args(&args)
            .output()
            .await
            .map_err(|e| tonic::Status::unavailable(format!("temporal cli unavailable: {e}")))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            let detail = if !stdout.trim().is_empty() {
                stdout.to_string()
            } else {
                stderr.to_string()
            };
            return Err(map_temporal_error(&detail));
        }

        Ok(SignalWorkflowExecutionResponse {})
    }
}

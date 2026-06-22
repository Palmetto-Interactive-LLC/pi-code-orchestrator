pub mod activities;
pub mod client;
pub mod signals;
pub mod worker;

pub use worker::Worker;

use sqlx::SqlitePool;
use tracing::info;

/// Convenience wrapper to create and run a Temporal worker.
/// Connects to `addr` and polls `task_queue` until `shutdown` fires.
pub async fn run_worker(
    db_pool: SqlitePool,
    addr: &str,
    task_queue: &str,
    shutdown: tokio::sync::watch::Receiver<bool>,
) -> anyhow::Result<()> {
    let mut worker = Worker::new(db_pool, addr, task_queue).await?;
    info!(addr = %addr, task_queue = %task_queue, "Temporal worker starting");
    worker.run(shutdown).await
}

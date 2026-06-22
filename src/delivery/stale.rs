use chrono::Utc;
use sqlx::SqlitePool;
use tracing::{info, warn};

use crate::db::queries::{get_stale_leases, log_event};
use crate::types::Lease;

/// Scan for leases that have been alive longer than 5 minutes and escalate them.
pub async fn check_stale_assignments(pool: &SqlitePool) -> anyhow::Result<()> {
    let stale_leases = get_stale_leases(pool).await?;
    let now = Utc::now();

    for lease in stale_leases {
        let age = now - lease.created_at;
        if age.num_seconds() > 300 {
            warn!(
                lease_id = %lease.id,
                work_item_id = %lease.work_item_id,
                agent_id = %lease.agent_id,
                age_secs = age.num_seconds(),
                "Stale lease detected"
            );
            escalate_stale(pool, &lease).await?;
        }
    }

    Ok(())
}

/// Record stale assignment audit state; Temporal owns status and recovery decisions.
pub async fn escalate_stale(pool: &SqlitePool, lease: &Lease) -> anyhow::Result<()> {
    log_event(
        pool,
        "system",
        Some(&lease.agent_id),
        "escalate_stale",
        Some(&lease.work_item_id),
    )
    .await?;

    info!(
        work_item_id = %lease.work_item_id,
        agent_id = %lease.agent_id,
        "Escalated stale assignment to Temporal"
    );

    // Temporal signal placeholder — actual implementation would invoke the temporal client here.
    Ok(())
}

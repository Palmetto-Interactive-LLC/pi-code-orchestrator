//! Human intervention module for Lantern Relay.
//!
//! Human commands record local audit state in this Rust launcher. Runtime control
//! is moving to Temporal workflow signals.

pub mod commands;
pub mod detect;

use anyhow::{anyhow, Result};
use once_cell::sync::OnceCell;
use sqlx::SqlitePool;

static DB_POOL: OnceCell<SqlitePool> = OnceCell::new();

/// Bind the shared database pool so that human module operations can persist state.
pub fn set_db_pool(pool: SqlitePool) {
    let _ = DB_POOL.set(pool);
}

/// Access the shared database pool.
pub(crate) fn pool() -> Result<&'static SqlitePool> {
    DB_POOL
        .get()
        .ok_or_else(|| anyhow!("Database pool not initialized"))
}

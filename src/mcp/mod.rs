pub mod server;
pub mod tools;

use sqlx::SqlitePool;

/// Run the MCP stdio server with the given database pool.
pub async fn run_mcp_server(pool: SqlitePool) -> anyhow::Result<()> {
    let srv = server::McpServer::new(pool);
    srv.run().await
}

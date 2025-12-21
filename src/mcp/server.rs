//! MCP server implementation using rmcp StreamableHttpService

use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
use rmcp::transport::StreamableHttpService;
use sqlx::PgPool;

use super::quote_tools::QuoteTools;

/// Create MCP service
pub fn create_mcp_router(
    pool: PgPool,
    upload_dir: String,
    max_file_size: usize,
) -> StreamableHttpService<QuoteTools> {
    StreamableHttpService::new(
        move || Ok(QuoteTools::new(pool.clone(), upload_dir.clone(), max_file_size)),
        LocalSessionManager::default().into(),
        Default::default(),
    )
}

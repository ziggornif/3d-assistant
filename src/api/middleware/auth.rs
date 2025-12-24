use axum::{
    extract::Request,
    http::{StatusCode, header},
    middleware::Next,
    response::Response,
};

/// Simple admin token authentication middleware
/// Uses cookie-based auth for SSR admin interface
pub async fn admin_auth(request: Request, next: Next) -> Result<Response, StatusCode> {
    let admin_token =
        std::env::var("ADMIN_TOKEN").unwrap_or_else(|_| "admin-secret-token".to_string());

    // Cookie-based auth for SSR admin interface
    let cookie_header = request
        .headers()
        .get(header::COOKIE)
        .and_then(|h| h.to_str().ok());

    if let Some(cookies) = cookie_header {
        for cookie in cookies.split(';') {
            let cookie = cookie.trim();
            if let Some(value) = cookie.strip_prefix("admin_token=")
                && value == admin_token
            {
                return Ok(next.run(request).await);
            }
        }
    }

    tracing::warn!("Missing or invalid authentication (no valid cookie)");
    Err(StatusCode::UNAUTHORIZED)
}

/// MCP token authentication middleware
/// Uses Bearer token authentication for MCP clients (AI models, automation tools)
pub async fn mcp_auth(request: Request, next: Next) -> Result<Response, StatusCode> {
    let mcp_token = std::env::var("MCP_TOKEN").unwrap_or_else(|_| "mcp-secret-token".to_string());

    // Bearer token auth for MCP endpoint
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok());

    if let Some(auth) = auth_header
        && let Some(token) = auth.strip_prefix("Bearer ")
        && token == mcp_token
    {
        return Ok(next.run(request).await);
    }

    tracing::warn!("Missing or invalid MCP authentication (invalid or missing Bearer token)");
    Err(StatusCode::UNAUTHORIZED)
}

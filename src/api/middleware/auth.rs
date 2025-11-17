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

/// Extract admin username from request (for audit trail)
/// For MVP, returns a static admin user
#[allow(dead_code)]
pub fn get_admin_user(_request: &Request) -> String {
    "admin".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_admin_user_returns_admin() {
        let request = axum::http::Request::builder()
            .body(axum::body::Body::empty())
            .unwrap();
        let user = get_admin_user(&request);
        assert_eq!(user, "admin");
    }
}

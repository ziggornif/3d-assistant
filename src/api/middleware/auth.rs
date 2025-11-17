use axum::{
    extract::Request,
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};

/// Simple admin token authentication middleware
/// In production, this would use JWT or session-based auth
pub async fn admin_auth(request: Request, next: Next) -> Result<Response, StatusCode> {
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok());

    match auth_header {
        Some(auth) if auth.starts_with("Bearer ") => {
            let token = &auth[7..];

            // For MVP, use a simple token from environment
            // In production, this would validate JWT or session
            let admin_token =
                std::env::var("ADMIN_TOKEN").unwrap_or_else(|_| "admin-secret-token".to_string());

            if token == admin_token {
                Ok(next.run(request).await)
            } else {
                tracing::warn!("Invalid admin token provided");
                Err(StatusCode::UNAUTHORIZED)
            }
        }
        _ => {
            tracing::warn!("Missing or invalid Authorization header");
            Err(StatusCode::UNAUTHORIZED)
        }
    }
}

/// Extract admin username from request (for audit trail)
/// For MVP, returns a static admin user
#[allow(dead_code)]
pub fn get_admin_user(_request: &Request) -> String {
    "admin".to_string()
}

/// Extract token from Authorization header
#[allow(dead_code)]
pub fn extract_bearer_token(auth_header: &str) -> Option<&str> {
    if auth_header.starts_with("Bearer ") {
        Some(&auth_header[7..])
    } else {
        None
    }
}

/// Validate token against expected value
#[allow(dead_code)]
pub fn validate_token(token: &str, expected: &str) -> bool {
    token == expected
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_bearer_token_valid() {
        let header = "Bearer my-secret-token";
        let token = extract_bearer_token(header);
        assert_eq!(token, Some("my-secret-token"));
    }

    #[test]
    fn test_extract_bearer_token_empty_token() {
        let header = "Bearer ";
        let token = extract_bearer_token(header);
        assert_eq!(token, Some(""));
    }

    #[test]
    fn test_extract_bearer_token_no_bearer_prefix() {
        let header = "Basic my-token";
        let token = extract_bearer_token(header);
        assert_eq!(token, None);
    }

    #[test]
    fn test_extract_bearer_token_lowercase_bearer() {
        let header = "bearer my-token";
        let token = extract_bearer_token(header);
        assert_eq!(token, None);
    }

    #[test]
    fn test_validate_token_correct() {
        assert!(validate_token("admin-secret-token", "admin-secret-token"));
    }

    #[test]
    fn test_validate_token_incorrect() {
        assert!(!validate_token("wrong-token", "admin-secret-token"));
    }

    #[test]
    fn test_validate_token_empty() {
        assert!(!validate_token("", "admin-secret-token"));
    }

    #[test]
    fn test_get_admin_user_returns_admin() {
        let request = axum::http::Request::builder()
            .body(axum::body::Body::empty())
            .unwrap();
        let user = get_admin_user(&request);
        assert_eq!(user, "admin");
    }
}

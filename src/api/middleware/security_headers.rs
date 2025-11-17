use axum::{
    body::Body,
    extract::State,
    http::{header, HeaderName, HeaderValue, Request, Response},
    middleware::Next,
};

use crate::api::routes::AppState;

/// Security headers middleware
/// Adds essential security headers to all responses
pub async fn security_headers(
    State(state): State<AppState>,
    request: Request<Body>,
    next: Next,
) -> Response<Body> {
    let is_production = state.config.is_production();
    let mut response = next.run(request).await;
    let headers = response.headers_mut();

    // Prevent clickjacking attacks
    headers.insert(header::X_FRAME_OPTIONS, HeaderValue::from_static("DENY"));

    // Prevent MIME type sniffing
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );

    // Enforce HTTPS (will be ignored in development)
    // max-age=31536000 = 1 year
    headers.insert(
        header::STRICT_TRANSPORT_SECURITY,
        HeaderValue::from_static("max-age=31536000; includeSubDomains"),
    );

    // Control referrer information
    headers.insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static("strict-origin-when-cross-origin"),
    );

    // Content Security Policy
    // Allow self for scripts/styles, Three.js from unpkg, and inline scripts for SSR data
    let csp = if is_production {
        // Production: strict CSP with only 'self'
        "default-src 'self'; \
         script-src 'self' 'unsafe-inline' https://unpkg.com; \
         style-src 'self' 'unsafe-inline'; \
         img-src 'self' data: blob:; \
         connect-src 'self'; \
         font-src 'self'; \
         object-src 'none'; \
         frame-ancestors 'none'; \
         base-uri 'self'; \
         form-action 'self'"
    } else {
        // Development: allow localhost and 127.0.0.1
        "default-src 'self'; \
         script-src 'self' 'unsafe-inline' https://unpkg.com; \
         style-src 'self' 'unsafe-inline'; \
         img-src 'self' data: blob:; \
         connect-src 'self' http://localhost:* http://127.0.0.1:* https://localhost:* https://127.0.0.1:*; \
         font-src 'self'; \
         object-src 'none'; \
         frame-ancestors 'none'; \
         base-uri 'self'; \
         form-action 'self'"
    };

    headers.insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_str(csp).unwrap(),
    );

    // Permissions Policy (formerly Feature-Policy)
    // Restrict access to browser features
    headers.insert(
        HeaderName::from_static("permissions-policy"),
        HeaderValue::from_static(
            "accelerometer=(), camera=(), geolocation=(), gyroscope=(), magnetometer=(), microphone=(), payment=(), usb=()"
        ),
    );

    // Legacy XSS Protection (for older browsers)
    headers.insert(
        HeaderName::from_static("x-xss-protection"),
        HeaderValue::from_static("1; mode=block"),
    );

    response
}

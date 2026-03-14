use axum::{
    Router,
    extract::DefaultBodyLimit,
    middleware,
    routing::{delete, get, patch, post},
};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

use crate::api::handlers::{admin, auth, materials, quote, ssr, upload, user_quotes};
use crate::api::middleware::{
    admin_auth, create_login_rate_limiter, create_rate_limiter, mcp_auth, security_headers,
};
use crate::config::Config;
use crate::db::DbPool;

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub pool: DbPool,
    pub config: Arc<Config>,
}

/// Create the main application router
pub fn create_router(pool: DbPool, config: Config) -> Router {
    let upload_dir = config.upload_dir.clone();
    let static_dir = config.static_dir.clone();
    let max_file_size = config.max_file_size_bytes as usize;

    let state = AppState {
        pool: pool.clone(),
        config: Arc::new(config),
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        // SSR Pages (Server-Side Rendered)
        .route("/", get(ssr::index_page))
        .route("/login", get(ssr::login_page))
        .route("/register", get(ssr::register_page))
        .route("/my-quotes", get(ssr::my_quotes_page))
        .route("/my-quotes/{quote_id}", get(ssr::quote_detail_page))
        .route("/admin", get(ssr::admin_page))
        .route(
            "/admin/login",
            post(ssr::admin_login).layer(create_login_rate_limiter()),
        )
        .route("/admin/logout", post(ssr::admin_logout))
        // Authentication API
        .route(
            "/api/auth/register",
            post(auth::register).layer(create_login_rate_limiter()),
        )
        .route(
            "/api/auth/login",
            post(auth::login).layer(create_login_rate_limiter()),
        )
        .route("/api/auth/logout", post(auth::logout))
        .route("/api/auth/me", get(auth::me))
        // User quotes (authenticated)
        .route("/api/users/me/quotes", get(user_quotes::list_my_quotes))
        .route(
            "/api/users/me/quotes/{quote_id}",
            get(user_quotes::get_my_quote),
        )
        .route(
            "/api/users/me/quotes/{quote_id}",
            patch(user_quotes::soft_delete_quote),
        )
        .route(
            "/api/users/me/quotes/{quote_id}/export",
            get(user_quotes::export_quote),
        )
        .route(
            "/api/users/me/quotes/{quote_id}/recalculate",
            post(user_quotes::recalculate_quote),
        )
        // Session management
        .route("/api/sessions", post(upload::create_session))
        // File upload
        .route(
            "/api/sessions/{session_id}/models",
            post(upload::upload_model),
        )
        .route(
            "/api/sessions/{session_id}/models/{model_id}",
            delete(upload::delete_model),
        )
        .route(
            "/api/sessions/{session_id}/models",
            get(upload::get_session_models),
        )
        // Materials
        .route("/api/materials", get(materials::list_materials))
        // Model configuration
        .route(
            "/api/sessions/{session_id}/models/{model_id}",
            patch(quote::configure_model),
        )
        // Quote generation
        .route(
            "/api/sessions/{session_id}/quote",
            post(quote::generate_quote),
        )
        .route(
            "/api/sessions/{session_id}/quote",
            get(quote::get_current_quote),
        )
        // Admin endpoints (protected by auth middleware)
        .nest(
            "/api/admin",
            Router::new()
                .route("/materials", get(admin::list_materials))
                .route("/materials", post(admin::create_material))
                .route("/materials/{id}", patch(admin::update_material))
                .route("/pricing-history", get(admin::get_pricing_history))
                .route("/cleanup", post(admin::cleanup_expired_sessions))
                .route("/users", get(admin::list_users))
                .route("/users/{user_id}", patch(admin::update_user_status))
                .layer(middleware::from_fn(admin_auth)),
        )
        // MCP (Model Context Protocol) endpoint with authentication
        .nest(
            "/mcp",
            Router::new()
                .fallback_service(crate::mcp::create_mcp_router(
                    pool,
                    upload_dir.clone(),
                    max_file_size,
                ))
                .layer(middleware::from_fn(mcp_auth)),
        )
        // Serve uploaded files
        .nest_service("/uploads", ServeDir::new(upload_dir))
        // Serve static frontend assets (CSS, JS, images)
        .nest_service("/static", ServeDir::new(static_dir))
        // Health check
        .route("/health", get(health_check))
        .layer(DefaultBodyLimit::max(100 * 1024 * 1024)) // 100MB limit for file uploads
        .layer(create_rate_limiter()) // Global rate limiting: 20 req/s, burst 100
        .layer(middleware::from_fn_with_state(state.clone(), security_headers)) // Security headers
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state)
}

/// Health check endpoint
async fn health_check() -> &'static str {
    "OK"
}

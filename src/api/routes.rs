use axum::{
    routing::{get, post, patch, delete},
    Router,
    extract::DefaultBodyLimit,
    middleware,
};
use std::sync::Arc;
use tower_http::cors::{CorsLayer, Any};
use tower_http::trace::TraceLayer;
use tower_http::services::ServeDir;

use crate::api::handlers::{upload, quote, materials, admin, ssr};
use crate::api::middleware::{admin_auth, create_rate_limiter};
use crate::db::DbPool;
use crate::config::Config;

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

    let state = AppState {
        pool,
        config: Arc::new(config),
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        // SSR Pages (Server-Side Rendered)
        .route("/", get(ssr::index_page))
        .route("/admin", get(ssr::admin_page))
        .route("/admin/login", post(ssr::admin_login))
        .route("/admin/logout", post(ssr::admin_logout))
        // Session management
        .route("/api/sessions", post(upload::create_session))
        // File upload
        .route(
            "/api/sessions/:session_id/models",
            post(upload::upload_model),
        )
        .route(
            "/api/sessions/:session_id/models/:model_id",
            delete(upload::delete_model),
        )
        // Materials
        .route("/api/materials", get(materials::list_materials))
        // Model configuration
        .route(
            "/api/sessions/:session_id/models/:model_id",
            patch(quote::configure_model),
        )
        // Quote generation
        .route(
            "/api/sessions/:session_id/quote",
            post(quote::generate_quote),
        )
        .route(
            "/api/sessions/:session_id/quote",
            get(quote::get_current_quote),
        )
        // Admin endpoints (protected by auth middleware)
        .nest(
            "/api/admin",
            Router::new()
                .route("/materials", get(admin::list_materials))
                .route("/materials", post(admin::create_material))
                .route("/materials/:id", patch(admin::update_material))
                .route("/pricing-history", get(admin::get_pricing_history))
                .route("/cleanup", post(admin::cleanup_expired_sessions))
                .layer(middleware::from_fn(admin_auth)),
        )
        // Serve uploaded files
        .nest_service("/uploads", ServeDir::new(upload_dir))
        // Serve static frontend assets (CSS, JS, images)
        .nest_service("/static", ServeDir::new(static_dir))
        // Health check
        .route("/health", get(health_check))
        .layer(DefaultBodyLimit::max(100 * 1024 * 1024)) // 100MB limit for file uploads
        .layer(create_rate_limiter()) // Global rate limiting: 20 req/s, burst 100
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state)
}

/// Health check endpoint
async fn health_check() -> &'static str {
    "OK"
}

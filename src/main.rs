mod api;
mod business;
mod config;
mod db;
mod integrations;
mod mcp;
mod models;
mod persistence;

use anyhow::Result;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize structured logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,tower_http=debug,axum::rejection=trace".into()),
        )
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    tracing::info!("Starting 3D Quote Service");

    // Load configuration
    let config = config::Config::from_env()?;
    tracing::info!("Configuration loaded: {}:{}", config.host, config.port);

    // Ensure upload directory exists
    std::fs::create_dir_all(&config.upload_dir)?;
    tracing::info!("Upload directory: {}", config.upload_dir);

    // Initialize templates for SSR
    business::init_templates(&config.template_dir).expect("Failed to load templates");
    tracing::info!("Templates loaded from: {}", config.template_dir);

    // Initialize database
    let pool = db::init_pool(&config.database_url).await?;
    tracing::info!("Database connection pool initialized");

    // Run migrations
    db::run_migrations(&pool).await?;
    tracing::info!("Database migrations completed");

    // Seed initial data
    db::seed_data(&pool).await?;
    tracing::info!("Seed data loaded");

    // Cleanup expired sessions on startup
    let session_service = business::SessionService::new(pool.clone(), &config.upload_dir);
    match session_service.cleanup_expired().await {
        Ok(result) => {
            tracing::info!(
                "Startup cleanup: {} expired sessions deleted, {} directories removed",
                result.sessions_deleted,
                result.directories_deleted
            );
            if !result.errors.is_empty() {
                tracing::warn!("Cleanup errors: {:?}", result.errors);
            }
        }
        Err(e) => {
            tracing::warn!("Failed to cleanup expired sessions on startup: {}", e);
        }
    }

    // Create router
    let app = api::create_router(pool, config.clone());

    // Start server
    let addr = format!("{}:{}", config.host, config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("Server listening on {}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}

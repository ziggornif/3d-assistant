mod api;
mod config;
mod db;
mod models;
mod services;

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
    tracing::info!(
        "Configuration loaded: {}:{}",
        config.host,
        config.port
    );

    // Ensure upload directory exists
    std::fs::create_dir_all(&config.upload_dir)?;
    tracing::info!("Upload directory: {}", config.upload_dir);

    // Initialize templates for SSR
    services::init_templates(&config.template_dir)
        .expect("Failed to load templates");
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

    // Create router
    let app = api::create_router(pool, config.clone());

    // Start server
    let addr = format!("{}:{}", config.host, config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("Server listening on {}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}

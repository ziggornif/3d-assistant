mod api;
mod business;
mod config;
mod db;
mod integrations;
mod mcp;
mod models;
mod observability;
mod persistence;

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Load configuration first (needed for observability setup)
    let config = config::Config::from_env()?;

    // Validate configuration (fail fast in production)
    config.validate()?;

    // Initialize OpenTelemetry tracing
    observability::init_tracing(
        &config.otel_exporter_otlp_endpoint,
        &config.otel_service_name,
        &config.environment,
    )?;

    // Initialize metrics
    let metrics = observability::init_metrics(
        &config.otel_exporter_otlp_endpoint,
        &config.otel_service_name,
        &config.environment,
    )?;

    // Initialize structured logging with OpenTelemetry integration
    observability::init_logging(config.is_production())?;

    tracing::info!("Starting 3D Quote Service");
    tracing::info!(
        "Configuration loaded: {}:{} (environment: {})",
        config.host,
        config.port,
        config.environment
    );
    tracing::info!(
        "OpenTelemetry initialized: endpoint={}",
        config.otel_exporter_otlp_endpoint
    );

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

    // Graceful shutdown: flush telemetry before exit
    tracing::info!("Shutting down gracefully");
    observability::shutdown_tracing();

    Ok(())
}

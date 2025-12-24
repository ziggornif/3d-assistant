//! Structured logging configuration with trace context
//!
//! This module configures environment-dependent logging:
//! - Development: Human-readable pretty format
//! - Production: JSON format with trace context

use anyhow::Result;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};

/// Initialize structured logging based on environment
///
/// - Development: Pretty-printed human-readable logs
/// - Production: JSON logs with trace_id, span_id, and structured fields
///
/// # Arguments
/// * `is_production` - Whether running in production mode
pub fn init_logging(is_production: bool) -> Result<()> {
    // Create OpenTelemetry tracing layer using the global tracer
    let telemetry_layer = tracing_opentelemetry::layer();

    // Create environment filter
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "info,tower_http=debug,axum::rejection=trace".into());

    if is_production {
        // Production: JSON format with trace context
        Registry::default()
            .with(env_filter)
            .with(telemetry_layer)
            .with(
                tracing_subscriber::fmt::layer()
                    .json()
                    .with_current_span(true) // Include trace context
                    .with_span_list(false)
                    .with_target(true),
            )
            .init();
    } else {
        // Development: Pretty format for readability
        Registry::default()
            .with(env_filter)
            .with(telemetry_layer)
            .with(
                tracing_subscriber::fmt::layer()
                    .pretty()
                    .with_target(true)
                    .with_thread_ids(false)
                    .with_thread_names(false),
            )
            .init();
    }

    Ok(())
}

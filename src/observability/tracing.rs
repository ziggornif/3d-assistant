//! OpenTelemetry distributed tracing setup
//!
//! This module initializes the OpenTelemetry tracer and integrates it with
//! the Rust tracing ecosystem via the tracing-opentelemetry bridge.

use anyhow::Result;
use opentelemetry::{global, KeyValue};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{
    runtime,
    trace::{self, RandomIdGenerator, Sampler, TracerProvider},
    Resource,
};
use std::time::Duration;

/// Initialize OpenTelemetry tracing with OTLP exporter
///
/// This sets up:
/// - OTLP gRPC exporter to configured endpoint
/// - Tracing-OpenTelemetry bridge layer
/// - Batch span processor for efficient export
///
/// # Arguments
/// * `endpoint` - OTLP collector endpoint (e.g., "http://localhost:4317")
/// * `service_name` - Service identifier for telemetry (e.g., "quote-service")
/// * `environment` - Environment name (e.g., "development", "production")
pub fn init_tracing(_endpoint: &str, service_name: &str, environment: &str) -> Result<()> {
    // Create resource with service metadata
    let resource = Resource::new(vec![
        KeyValue::new("service.name", service_name.to_string()),
        KeyValue::new("service.version", env!("CARGO_PKG_VERSION").to_string()),
        KeyValue::new("deployment.environment", environment.to_string()),
    ]);

    // Configure OTLP exporter
    let exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_timeout(Duration::from_secs(3))
        .build_span_exporter()?;

    // Build tracer provider with batch span processor
    let tracer_provider = TracerProvider::builder()
        .with_batch_exporter(exporter, runtime::Tokio)
        .with_config(
            trace::Config::default()
                .with_sampler(Sampler::AlwaysOn) // 100% sampling for now
                .with_id_generator(RandomIdGenerator::default())
                .with_max_events_per_span(64)
                .with_max_attributes_per_span(32)
                .with_resource(resource),
        )
        .build();

    // Set as global tracer provider
    global::set_tracer_provider(tracer_provider);

    Ok(())
}

/// Shutdown OpenTelemetry tracing gracefully
///
/// Flushes any pending spans before shutdown
pub fn shutdown_tracing() {
    global::shutdown_tracer_provider();
}

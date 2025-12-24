//! OpenTelemetry metrics for business and technical monitoring
//!
//! This module defines and exports metrics including:
//! - Business metrics: quotes generated, file sizes, calculation times
//! - Technical metrics: HTTP request rates, database pool stats, error rates

use anyhow::Result;
use opentelemetry::{global, KeyValue};
use opentelemetry_sdk::{
    metrics::SdkMeterProvider,
    Resource,
};
use std::sync::Arc;

/// Metrics registry for the application
pub struct Metrics {
    // Business metrics - counters
    pub quotes_generated_total: Arc<opentelemetry::metrics::Counter<u64>>,
    pub models_uploaded_total: Arc<opentelemetry::metrics::Counter<u64>>,

    // Business metrics - histograms
    pub file_upload_size_bytes: Arc<opentelemetry::metrics::Histogram<f64>>,
    pub quote_calculation_duration_ms: Arc<opentelemetry::metrics::Histogram<f64>>,
    pub volume_calculation_duration_ms: Arc<opentelemetry::metrics::Histogram<f64>>,

    // Technical metrics - counters
    pub http_requests_total: Arc<opentelemetry::metrics::Counter<u64>>,
    pub db_errors_total: Arc<opentelemetry::metrics::Counter<u64>>,

    // Technical metrics - histograms
    pub http_request_duration_ms: Arc<opentelemetry::metrics::Histogram<f64>>,
    pub db_query_duration_ms: Arc<opentelemetry::metrics::Histogram<f64>>,

    // Technical metrics - gauges (via UpDownCounter)
    pub db_connections_active: Arc<opentelemetry::metrics::UpDownCounter<i64>>,
    pub http_active_requests: Arc<opentelemetry::metrics::UpDownCounter<i64>>,
}

impl Metrics {
    /// Create a new metrics registry with all metrics initialized
    fn new(meter: opentelemetry::metrics::Meter) -> Self {
        // Business metrics
        let quotes_generated_total = Arc::new(
            meter
                .u64_counter("quotes_generated_total")
                .with_description("Total number of quotes generated")
                .init(),
        );

        let models_uploaded_total = Arc::new(
            meter
                .u64_counter("models_uploaded_total")
                .with_description("Total number of 3D models uploaded")
                .init(),
        );

        let file_upload_size_bytes = Arc::new(
            meter
                .f64_histogram("file_upload_size_bytes")
                .with_description("Distribution of uploaded file sizes in bytes")
                .init(),
        );

        let quote_calculation_duration_ms = Arc::new(
            meter
                .f64_histogram("quote_calculation_duration_ms")
                .with_description("Time to calculate quote in milliseconds")
                .init(),
        );

        let volume_calculation_duration_ms = Arc::new(
            meter
                .f64_histogram("volume_calculation_duration_ms")
                .with_description("Time to parse file and calculate volume in milliseconds")
                .init(),
        );

        // Technical metrics
        let http_requests_total = Arc::new(
            meter
                .u64_counter("http_requests_total")
                .with_description("Total HTTP requests received")
                .init(),
        );

        let db_errors_total = Arc::new(
            meter
                .u64_counter("db_errors_total")
                .with_description("Total database errors encountered")
                .init(),
        );

        let http_request_duration_ms = Arc::new(
            meter
                .f64_histogram("http_request_duration_ms")
                .with_description("HTTP request latency in milliseconds")
                .init(),
        );

        let db_query_duration_ms = Arc::new(
            meter
                .f64_histogram("db_query_duration_ms")
                .with_description("Database query latency in milliseconds")
                .init(),
        );

        let db_connections_active = Arc::new(
            meter
                .i64_up_down_counter("db_connections_active")
                .with_description("Number of active database connections")
                .init(),
        );

        let http_active_requests = Arc::new(
            meter
                .i64_up_down_counter("http_active_requests")
                .with_description("Number of currently processing HTTP requests")
                .init(),
        );

        Self {
            quotes_generated_total,
            models_uploaded_total,
            file_upload_size_bytes,
            quote_calculation_duration_ms,
            volume_calculation_duration_ms,
            http_requests_total,
            db_errors_total,
            http_request_duration_ms,
            db_query_duration_ms,
            db_connections_active,
            http_active_requests,
        }
    }
}

/// Initialize OpenTelemetry metrics with OTLP exporter
///
/// This sets up:
/// - OTLP metrics exporter
/// - Business metrics (quotes, files, pricing)
/// - Technical metrics (HTTP, database, errors)
///
/// # Arguments
/// * `endpoint` - OTLP collector endpoint (e.g., "http://localhost:4317")
/// * `service_name` - Service identifier for telemetry
/// * `environment` - Environment name
pub fn init_metrics(_endpoint: &str, service_name: &str, environment: &str) -> Result<Metrics> {
    // For now, use a simple in-memory meter provider
    // TODO: Add OTLP metrics export once API is stable
    let resource = Resource::new(vec![
        KeyValue::new("service.name", service_name.to_string()),
        KeyValue::new("service.version", env!("CARGO_PKG_VERSION").to_string()),
        KeyValue::new("deployment.environment", environment.to_string()),
    ]);

    let meter_provider = SdkMeterProvider::builder()
        .with_resource(resource)
        .build();

    global::set_meter_provider(meter_provider);

    // Create meter and initialize metrics
    let meter = global::meter("quote-service");
    let metrics = Metrics::new(meter);

    Ok(metrics)
}

/// Shutdown OpenTelemetry metrics gracefully
pub fn shutdown_metrics() {
    // Metrics shutdown is handled by global shutdown
}

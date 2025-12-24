//! Observability module for OpenTelemetry instrumentation
//!
//! This module provides comprehensive observability capabilities including:
//! - Distributed tracing with OpenTelemetry
//! - Business and technical metrics
//! - Structured logging with trace context

pub mod logging;
pub mod metrics;
pub mod tracing;

// Re-export commonly used items
pub use self::logging::init_logging;
pub use self::metrics::{init_metrics, Metrics};
pub use self::tracing::{init_tracing, shutdown_tracing};

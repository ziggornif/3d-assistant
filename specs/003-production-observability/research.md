# Research: Production Deployment and Observability

**Feature**: 003-production-observability
**Date**: 2025-12-24
**Status**: Completed

## Overview

This document consolidates research findings for implementing production deployment and OpenTelemetry observability in the 3D print quote service (Rust/Axum backend).

## Decision 1: OpenTelemetry SDK and Integration

**Question**: Which OpenTelemetry crates should we use for Rust, and how do we integrate with existing `tracing`?

**Decision**: Use `opentelemetry` (0.21+) with `tracing-opentelemetry` bridge

**Rationale**:
- The Rust `tracing` ecosystem is already integrated throughout the codebase
- `tracing-opentelemetry` provides zero-friction bridge between `tracing` spans/events and OpenTelemetry
- No need to refactor existing instrumentation - existing `#[instrument]` macros automatically become OTel traces
- Standard approach in Rust ecosystem (used by AWS, Cloudflare, etc.)

**Alternatives Considered**:
1. **Direct OpenTelemetry SDK without tracing**: Rejected - would require refactoring all existing instrumentation
2. **Custom telemetry solution**: Rejected - reinventing the wheel, no vendor portability
3. **Prometheus only**: Rejected - doesn't provide distributed tracing or structured logs

**Implementation Approach**:
```rust
// Existing code continues to work:
#[tracing::instrument]
async fn process_file() { ... }

// New: OpenTelemetry layer added to tracing subscriber
use tracing_subscriber::{layer::SubscriberExt, Registry};
use tracing_opentelemetry::OpenTelemetryLayer;

let tracer = opentelemetry_otlp::new_pipeline()
    .tracing()
    .with_exporter(opentelemetry_otlp::new_exporter().tonic())
    .install_batch(opentelemetry::runtime::Tokio)?;

let telemetry = OpenTelemetryLayer::new(tracer);
let subscriber = Registry::default().with(telemetry);
tracing::subscriber::set_global_default(subscriber)?;
```

## Decision 2: Observability Backend Selection

**Question**: Should we use SigNoz, Grafana stack (Tempo+Loki+Prometheus), or another backend?

**Decision**: Primary: **SigNoz** (all-in-one). Alternative: **Grafana stack** documented for advanced users

**Rationale**:
- **SigNoz wins for simplicity**: Single Docker Compose, one UI for traces/metrics/logs, easier onboarding
- Grafana stack offers more flexibility but higher complexity (4 services: Tempo, Loki, Prometheus, Grafana)
- For MVP and small deployments, SigNoz reduces operational overhead
- Both are vendor-neutral (can switch backends via OTLP endpoint config)

**Alternatives Considered**:
1. **Jaeger**: Only traces, no metrics or logs - too limited
2. **Commercial SaaS (Honeycomb, Datadog)**: Higher cost, vendor lock-in, not self-hosted
3. **Prometheus + Jaeger** separately: More complex than Grafana stack, no unified UI

**SigNoz Docker Compose Structure**:
```yaml
services:
  clickhouse:  # SigNoz data store
  otel-collector:  # Receives OTLP exports
  query-service:  # SigNoz query backend
  frontend:  # SigNoz UI
  alertmanager:  # Optional alerts
```

**Resource Requirements**:
- Minimum: 2 CPU, 4GB RAM
- Recommended: 4 CPU, 8GB RAM for production

## Decision 3: Structured Logging Format

**Question**: How should we format logs for production (JSON vs human-readable)?

**Decision**: **Environment-dependent**: JSON in production, human-readable in development

**Rationale**:
- JSON logs are machine-parsable (essential for log aggregation tools like Loki)
- Human-readable logs improve local development experience
- `tracing-subscriber` supports both via `fmt::layer()` configuration

**Implementation**:
```rust
let log_format = match env::var("ENVIRONMENT") {
    Ok(env) if env == "production" => {
        tracing_subscriber::fmt::layer()
            .json()
            .with_current_span(true)  // Include trace context
            .with_target(false)
    },
    _ => {
        tracing_subscriber::fmt::layer()
            .pretty()
            .with_target(true)
    }
};
```

**JSON Log Fields**:
- `timestamp`: ISO 8601 UTC
- `level`: ERROR, WARN, INFO, DEBUG, TRACE
- `message`: Log message
- `trace_id`: OpenTelemetry trace ID (for correlation)
- `span_id`: Current span ID
- `service.name`: "quote-service"
- `fields`: Any structured data from the span

## Decision 4: Health Check Implementation

**Question**: What should `/health` and `/ready` endpoints check?

**Decision**:
- **`/health`** (liveness): Minimal check - just return 200 if service is running
- **`/ready`** (readiness): Check database connectivity + filesystem writability

**Rationale**:
- Liveness probes should be fast and lightweight (no dependencies)
- Readiness probes verify service can handle requests (dependencies healthy)
- Kubernetes/Docker healthcheck best practices

**Implementation Strategy**:
```rust
// /health - Always returns 200 if process is alive
async fn health_check() -> StatusCode {
    StatusCode::OK
}

// /ready - Checks dependencies
async fn readiness_check(pool: PgPool, upload_dir: PathBuf) -> Result<Json<ReadinessResponse>, StatusCode> {
    let mut checks = vec![];

    // Check database
    let db_check = sqlx::query("SELECT 1").fetch_one(&pool).await;
    checks.push(("database", db_check.is_ok()));

    // Check filesystem
    let fs_check = tokio::fs::File::create(upload_dir.join(".healthcheck")).await;
    checks.push(("filesystem", fs_check.is_ok()));

    let all_healthy = checks.iter().all(|(_, ok)| *ok);
    let status = if all_healthy { StatusCode::OK } else { StatusCode::SERVICE_UNAVAILABLE };

    Ok((status, Json(ReadinessResponse { checks })))
}
```

## Decision 5: Deployment Platform Strategy

**Question**: Which deployment platforms should we officially support?

**Decision**: Support 3 deployment methods with documentation:
1. **Docker Compose** (easiest, good for small-medium deployments)
2. **Ubuntu VPS with systemd** (full control, custom infrastructure)
3. **CleverCloud** (managed PaaS, if applicable to user's preference)

**Rationale**:
- Docker Compose covers 80% of use cases (dev, staging, small prod)
- VPS deployment provides learning path and full control
- Managed platforms (CleverCloud) reduce operational burden
- All three use same environment variable configuration (portable)

**Deployment Automation Scope**:
- Docker Compose: Fully automated (docker-compose.prod.yml)
- VPS: Semi-automated (scripts + manual nginx config)
- CleverCloud: Documentation only (platform-specific)

## Decision 6: Secrets Management

**Question**: How should we handle secrets in different environments?

**Decision**: **Environment variables only**, with validation at startup

**Rationale**:
- 12-factor app principle (config in environment)
- Works across all deployment platforms
- Clear separation between code and config
- Easy to rotate secrets without code changes

**Required Secrets**:
- `DATABASE_URL`: PostgreSQL connection string
- `ADMIN_TOKEN`: Admin interface authentication
- `MCP_TOKEN`: MCP API authentication
- `OTEL_EXPORTER_OTLP_ENDPOINT`: OpenTelemetry collector endpoint (optional, defaults to localhost:4317)

**Validation Strategy**:
- Fail fast at startup if required secrets missing
- Log clear error messages (without exposing secret values)
- Provide `.env.example` with placeholders

## Decision 7: Performance Overhead Mitigation

**Question**: How do we minimize OpenTelemetry performance impact?

**Decision**: Use batch export with conservative defaults

**Rationale**:
- Batching reduces network overhead (multiple spans per export)
- Tokio async runtime handles background export without blocking requests
- Conservative defaults prevent overwhelming collector

**Configuration**:
```rust
let tracer = opentelemetry_otlp::new_pipeline()
    .tracing()
    .with_exporter(
        opentelemetry_otlp::new_exporter()
            .tonic()
            .with_timeout(Duration::from_secs(3))
    )
    .with_trace_config(
        opentelemetry::sdk::trace::config()
            .with_max_events_per_span(64)
            .with_max_attributes_per_span(32)
            .with_max_links_per_span(32)
    )
    .install_batch(opentelemetry::runtime::Tokio)?;
```

**Sampling Strategy** (future consideration):
- Start with 100% sampling (< 1000 req/min expected)
- Add tail-based sampling if traffic grows
- Always sample errors (even if other traces dropped)

## Decision 8: Observability Stack Resource Limits

**Question**: What resource limits should we set for Docker Compose observability stack?

**Decision**: Set reasonable limits to prevent resource exhaustion

**Docker Compose Configuration**:
```yaml
services:
  clickhouse:
    deploy:
      resources:
        limits:
          cpus: '2'
          memory: 4G
        reservations:
          memory: 2G

  otel-collector:
    deploy:
      resources:
        limits:
          cpus: '1'
          memory: 1G
```

**Rationale**:
- Prevents single service from consuming all host resources
- Provides predictable behavior under load
- Can be adjusted based on actual usage patterns

## Key Takeaways

1. **Leverage existing `tracing` infrastructure** - OpenTelemetry integration is additive, not disruptive
2. **SigNoz for simplicity** - Single UI beats complexity for MVP
3. **Environment-based configuration** - Same code, different behavior via env vars
4. **Health checks follow Kubernetes conventions** - `/health` for liveness, `/ready` for readiness
5. **Multi-platform deployment** - Docker Compose primary, VPS documented, managed platforms optional
6. **Secrets via environment variables** - 12-factor app compliance, works everywhere
7. **Batch export for performance** - Minimize overhead while maintaining observability
8. **Resource limits prevent outages** - Controlled resource usage in Docker

## References

- [OpenTelemetry Rust SDK](https://github.com/open-telemetry/opentelemetry-rust)
- [tracing-opentelemetry integration](https://github.com/tokio-rs/tracing-opentelemetry)
- [SigNoz Documentation](https://signoz.io/docs/)
- [Grafana Observability Stack](https://grafana.com/oss/)
- [12-Factor App: Config](https://12factor.net/config)
- [Kubernetes Health Checks](https://kubernetes.io/docs/tasks/configure-pod-container/configure-liveness-readiness-startup-probes/)

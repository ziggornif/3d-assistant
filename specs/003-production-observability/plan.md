# Implementation Plan: Production Deployment and Observability

**Branch**: `003-production-observability` | **Date**: 2025-12-24 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/003-production-observability/spec.md`

## Summary

This feature prepares the 3D print quote service for production deployment with comprehensive observability using OpenTelemetry. The implementation focuses on secure configuration management, distributed tracing, metrics collection, structured logging, and deployment automation. The technical approach leverages Rust's `tracing` ecosystem integrated with OpenTelemetry SDK, Docker Compose for stack deployment, and provides deployment guides for multiple platforms (VPS, Docker, managed services).

## Technical Context

**Language/Version**: Rust 1.75+ (current project standard)
**Primary Dependencies**:
- `opentelemetry` (0.21+) - OpenTelemetry SDK for Rust
- `opentelemetry-otlp` - OTLP exporter
- `tracing-opentelemetry` - Bridge between tracing and OpenTelemetry
- `tracing-subscriber` - Subscriber layer for structured logging
- `serde_json` - JSON log formatting
- Existing: `axum`, `sqlx`, `tokio`, `tracing`

**Storage**: PostgreSQL 14+ (existing), File system for uploads (existing)
**Testing**: `cargo test` for unit/integration, Playwright for E2E (existing)
**Target Platform**: Linux servers (Ubuntu 22.04+), Docker containers, managed platforms (CleverCloud)
**Project Type**: Single Rust backend service (existing monolith)
**Performance Goals**:
- OpenTelemetry overhead < 5% latency impact
- Health checks respond < 100ms
- Telemetry export failures don't block requests
- Deployment time < 1 hour for new environments

**Constraints**:
- Must not break existing functionality
- Telemetry must be non-intrusive (graceful degradation if collector unavailable)
- Production secrets must never appear in logs/traces
- Configuration must work across platforms (Docker, bare metal, PaaS)

**Scale/Scope**:
- Initial: < 1000 requests/minute
- Telemetry retention: 7 days default (configurable)
- Max trace size: 10MB per request
- Deployment targets: 1-3 environments (dev, staging, prod)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### Code Quality First
- ✅ **Static Analysis**: OpenTelemetry instrumentation will be added with zero warnings policy enforced
- ✅ **Type Safety**: All telemetry types strongly typed (Rust's type system enforced)
- ✅ **Documentation**: All new public APIs (health endpoints, telemetry helpers) will have docstrings
- ✅ **Single Responsibility**: Configuration management, telemetry, and health checks are separate modules
- ✅ **No Magic Numbers**: All thresholds (timeouts, buffer sizes) will be named constants

### Testing Standards
- ✅ **Coverage Threshold**: Target 80%+ for new code (configuration, health checks, telemetry setup)
- ✅ **Test Types**:
  - Unit tests for configuration loading, health check logic
  - Integration tests for telemetry export (with mock collector)
  - E2E tests for deployment scenarios
- ✅ **CI/CD Gate**: All tests must pass before merge
- ⚠️ **Note**: OpenTelemetry instrumentation is primarily infrastructure code; some paths (e.g., collector communication) may be tested via integration tests rather than pure unit tests

### User Experience Consistency
- ✅ **Response Time**: Health endpoints < 100ms (constitution: < 200ms)
- ✅ **Error Messages**: Configuration errors provide clear actionable messages
- ✅ **Graceful Degradation**: Telemetry failures don't impact user requests
- N/A **Accessibility**: No UI changes in this feature

### Performance Requirements
- ✅ **Response Time Targets**: OpenTelemetry overhead budgeted at < 5% p95 latency
- ✅ **Resource Constraints**: Telemetry buffer limits prevent memory leaks
- ✅ **Monitoring**: This feature IS the monitoring implementation
- ✅ **Performance Budgets**: Telemetry overhead measured in integration tests

**Constitution Compliance**: ✅ PASS - No violations. Standard infrastructure feature with testing appropriate to the domain.

## Project Structure

### Documentation (this feature)

```text
specs/003-production-observability/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0: OpenTelemetry decisions, deployment patterns
├── data-model.md        # Phase 1: N/A (no new data entities, telemetry is ephemeral)
├── quickstart.md        # Phase 1: Local testing with observability stack
├── contracts/           # Phase 1: Health endpoint specs, telemetry schema
│   ├── health.yaml      # OpenAPI spec for /health and /ready
│   └── telemetry.md     # Documented spans, metrics, log structure
└── tasks.md             # Phase 2: NOT created by /speckit.plan (use /speckit.tasks)
```

### Source Code (repository root)

```text
src/
├── config.rs                    # [ENHANCED] Add OTEL env vars, production mode
├── observability/               # [NEW] OpenTelemetry setup and utilities
│   ├── mod.rs
│   ├── tracing.rs              # Initialize OpenTelemetry tracer
│   ├── metrics.rs              # Business and technical metrics
│   └── logging.rs              # Structured JSON logging configuration
├── api/
│   ├── routes.rs               # [ENHANCED] Add /health and /ready endpoints
│   └── middleware/             # [ENHANCED] Trace propagation middleware
└── main.rs                     # [ENHANCED] Initialize observability on startup

docs/
├── deployment/                  # [NEW] Deployment guides
│   ├── docker-compose.md
│   ├── vps-ubuntu.md
│   └── clevercloud.md
└── observability/               # [NEW] Observability docs
    ├── dashboards.md
    └── troubleshooting.md

deployment/                      # [NEW] Deployment automation
├── docker-compose.prod.yml      # Production stack with observability
├── docker-compose.observability.yml  # Standalone observability stack
├── scripts/
│   ├── deploy-vps.sh
│   └── setup-db.sh
└── nginx/
    └── site.conf                # Reverse proxy configuration

Dockerfile                       # [ENHANCED] Add healthcheck

tests/
├── integration/
│   ├── observability_test.rs    # [NEW] Test telemetry export
│   └── health_checks_test.rs    # [NEW] Test health endpoints
└── e2e/
    └── deployment.spec.js       # [NEW] Test deployment scenarios
```

**Structure Decision**: Extending existing single Rust backend monolith. New `observability/` module for OpenTelemetry setup keeps concerns separated. Deployment artifacts live in new `/deployment` directory at repo root. Documentation enhanced with deployment and observability sections.

## Complexity Tracking

No constitution violations requiring justification. This is a standard infrastructure enhancement feature.

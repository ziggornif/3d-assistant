# Data Model: Production Deployment and Observability

**Feature**: 003-production-observability
**Date**: 2025-12-24

## Overview

This feature does not introduce new persistent data entities. It enhances the existing system with observability and deployment capabilities. Telemetry data (traces, metrics, logs) is ephemeral and managed by the observability backend (SigNoz/Grafana), not by the application database.

## Existing Entities (No Changes)

The following entities from the existing system are **unchanged** by this feature:

- **QuoteSession**: User quote sessions (existing)
- **UploadedModel**: 3D model files (existing)
- **Material**: Available printing materials (existing)
- **PricingHistory**: Historical pricing data (existing)

## Telemetry Data (Managed by Observability Backend)

These are **not** database entities, but data structures exported to the observability system:

### Trace Span

Represents a single operation within a distributed trace.

**Attributes**:
- `trace_id`: UUID - Unique identifier for the entire trace
- `span_id`: UUID - Unique identifier for this specific span
- `parent_span_id`: UUID | null - Parent span if nested
- `name`: string - Operation name (e.g., "process_file", "calculate_quote")
- `start_time`: timestamp - When operation started
- `end_time`: timestamp - When operation completed
- `status`: enum(OK, ERROR) - Operation outcome
- `attributes`: map - Custom key-value pairs (e.g., `file_size`, `material_id`)

**Example Trace**:
```
Request: Upload 3D file and generate quote
├─ Span: HTTP POST /api/sessions/{id}/upload
│  ├─ Span: validate_file (size, format)
│  ├─ Span: parse_stl_file
│  │  └─ Span: calculate_volume
│  ├─ Span: fetch_material (DB query)
│  └─ Span: calculate_price
└─ Response: 200 OK
```

### Metric

Represents a measurement over time.

**Types**:
1. **Counter**: Monotonically increasing value (e.g., total quotes generated)
2. **Gauge**: Current value that can go up or down (e.g., active database connections)
3. **Histogram**: Distribution of values (e.g., request latency percentiles)

**Business Metrics**:
- `quotes_generated_total`: Counter - Total quotes created
- `file_upload_size_bytes`: Histogram - Distribution of uploaded file sizes
- `quote_calculation_duration_ms`: Histogram - Time to calculate quote
- `material_usage_total`: Counter per material - Usage by material type

**Technical Metrics**:
- `http_requests_total`: Counter by endpoint and status - HTTP traffic
- `http_request_duration_ms`: Histogram by endpoint - Request latency
- `db_connections_active`: Gauge - Current database connections
- `db_query_duration_ms`: Histogram - Database query performance

### Log Entry

Structured event record.

**Fields**:
- `timestamp`: ISO 8601 UTC - When event occurred
- `level`: enum(ERROR, WARN, INFO, DEBUG, TRACE) - Severity
- `message`: string - Human-readable description
- `trace_id`: UUID | null - Link to trace (if within a request)
- `span_id`: UUID | null - Link to specific span
- `service.name`: string - Always "quote-service"
- `target`: string - Rust module path (e.g., "quote_service::business::pricing")
- `fields`: object - Structured data (varies by log type)

**Example Log** (JSON in production):
```json
{
  "timestamp": "2025-12-24T10:30:45.123Z",
  "level": "ERROR",
  "message": "Failed to parse STL file",
  "trace_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
  "span_id": "1234abcd",
  "service.name": "quote-service",
  "target": "quote_service::business::file_processor",
  "fields": {
    "error": "Invalid binary STL header",
    "file_name": "model.stl",
    "file_size_bytes": 1048576
  }
}
```

## Health Check Response

**Not persisted** - ephemeral response format for `/ready` endpoint.

**Structure**:
```rust
struct ReadinessResponse {
    status: String,           // "healthy" or "unhealthy"
    checks: Vec<HealthCheck>
}

struct HealthCheck {
    component: String,  // "database", "filesystem"
    status: String,     // "up", "down"
    message: Option<String>  // Error detail if down
}
```

**Example Response**:
```json
{
  "status": "healthy",
  "checks": [
    {
      "component": "database",
      "status": "up",
      "message": null
    },
    {
      "component": "filesystem",
      "status": "up",
      "message": null
    }
  ]
}
```

## Configuration Schema

**Not persisted** - loaded from environment variables at startup.

**New Environment Variables**:
- `ENVIRONMENT`: string (development | production) - Controls log format, telemetry sampling
- `OTEL_EXPORTER_OTLP_ENDPOINT`: string - OpenTelemetry collector endpoint (default: "http://localhost:4317")
- `OTEL_SERVICE_NAME`: string - Service identifier in traces (default: "quote-service")

**Existing Environment Variables** (no changes):
- `DATABASE_URL`, `ADMIN_TOKEN`, `MCP_TOKEN`, `HOST`, `PORT`, etc.

## Summary

This feature enhances observability **without introducing new database entities**. All telemetry data is managed by the observability backend (SigNoz/Grafana/etc.) and is not persisted in the application database. The existing data model remains unchanged.

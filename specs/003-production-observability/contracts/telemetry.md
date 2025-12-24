# Telemetry Contract: OpenTelemetry Spans, Metrics, and Logs

**Feature**: 003-production-observability
**Version**: 1.0.0
**Protocol**: OpenTelemetry Protocol (OTLP)

## Overview

This document defines the telemetry contract for the 3D print quote service. All telemetry data is exported via OTLP to a configurable collector endpoint.

## Service Attributes

All telemetry (traces, metrics, logs) includes these resource attributes:

| Attribute | Value | Description |
|-----------|-------|-------------|
| `service.name` | `quote-service` | Service identifier |
| `service.version` | `0.1.0` (from Cargo.toml) | Application version |
| `deployment.environment` | `$ENVIRONMENT` | development or production |

## Traces

### Automatic HTTP Instrumentation

All HTTP requests automatically generate traces via `tracing-opentelemetry`:

**Span Name**: `HTTP {method} {path_pattern}`

**Attributes**:
| Attribute | Type | Example | Description |
|-----------|------|---------|-------------|
| `http.method` | string | `POST` | HTTP method |
| `http.route` | string | `/api/sessions/:id/upload` | Route pattern |
| `http.status_code` | integer | `200` | Response status |
| `http.user_agent` | string | `curl/7.68.0` | User agent header |
| `http.client_ip` | string | `192.168.1.100` | Client IP address |

**Example Trace**:
```
Span: HTTP POST /api/sessions/:id/upload
├─ http.method: POST
├─ http.route: /api/sessions/:id/upload
├─ http.status_code: 200
└─ duration: 245ms
```

### Custom Business Spans

#### 1. File Upload Processing

**Span Name**: `process_uploaded_file`

**Attributes**:
| Attribute | Type | Example | Description |
|-----------|------|---------|-------------|
| `file.name` | string | `model.stl` | Original filename |
| `file.size_bytes` | integer | `1048576` | File size in bytes |
| `file.format` | string | `stl` or `3mf` | File format detected |
| `session.id` | string | `01HX...` | Quote session ID (ULID) |

**Child Spans**:
- `validate_file_format` - Check file type
- `validate_file_size` - Check size limits
- `parse_stl_file` or `parse_3mf_file` - Parse geometry
- `calculate_volume` - Compute 3D volume
- `save_to_filesystem` - Store uploaded file

#### 2. Quote Calculation

**Span Name**: `calculate_quote`

**Attributes**:
| Attribute | Type | Example | Description |
|-----------|------|---------|-------------|
| `session.id` | string | `01HX...` | Quote session ID |
| `model_count` | integer | `3` | Number of models in quote |
| `material.id` | string | `pla_standard` | Selected material |
| `total_volume_cm3` | float | `125.5` | Total volume across all models |
| `total_price` | float | `45.75` | Calculated total price |

**Child Spans**:
- `fetch_material_pricing` - Database query for material
- `compute_model_price` (per model) - Individual pricing
- `apply_minimum_order` - Business rule application

#### 3. Session Cleanup

**Span Name**: `cleanup_expired_sessions`

**Attributes**:
| Attribute | Type | Example | Description |
|-----------|------|---------|-------------|
| `sessions_deleted` | integer | `15` | Sessions cleaned up |
| `directories_deleted` | integer | `15` | Upload directories removed |
| `errors` | integer | `0` | Cleanup errors encountered |

**Child Spans**:
- `find_expired_sessions` - Database query
- `delete_session_files` (per session) - Filesystem cleanup
- `delete_session_data` - Database cleanup

#### 4. Database Operations

**Span Name**: `db.query`

**Attributes**:
| Attribute | Type | Example | Description |
|-----------|------|---------|-------------|
| `db.system` | string | `postgresql` | Database type |
| `db.statement` | string | `SELECT * FROM materials WHERE...` | SQL query (sanitized) |
| `db.operation` | string | `SELECT`, `INSERT`, `UPDATE`, `DELETE` | SQL operation |
| `db.rows_affected` | integer | `5` | Rows returned/affected |

**Note**: Query parameters are NOT included in `db.statement` to prevent leaking sensitive data.

## Metrics

### Business Metrics

#### Quote Generation

| Metric Name | Type | Labels | Description |
|-------------|------|--------|-------------|
| `quotes_generated_total` | Counter | `material_id` | Total quotes generated |
| `quote_value_total` | Counter | `material_id` | Sum of all quote prices |
| `models_uploaded_total` | Counter | `file_format` | Total 3D models uploaded |
| `file_upload_size_bytes` | Histogram | `file_format` | Distribution of file sizes |

**Example Usage**:
```rust
quotes_generated_total.add(1, &[("material_id", "pla_standard")]);
```

#### Quote Calculation Performance

| Metric Name | Type | Labels | Description |
|-------------|------|--------|-------------|
| `quote_calculation_duration_ms` | Histogram | `model_count_bucket` | Time to calculate quote |
| `volume_calculation_duration_ms` | Histogram | `file_format` | Time to parse and calculate volume |

**Buckets for `model_count_bucket`**:
- `1`: Single model quotes
- `2-5`: Small batch quotes
- `6-10`: Medium batch quotes
- `>10`: Large batch quotes

### Technical Metrics

#### HTTP Metrics

| Metric Name | Type | Labels | Description |
|-------------|------|--------|-------------|
| `http_requests_total` | Counter | `method`, `route`, `status` | Total HTTP requests |
| `http_request_duration_ms` | Histogram | `method`, `route` | Request latency |
| `http_active_requests` | Gauge | - | Currently processing requests |

**Label Values**:
- `method`: `GET`, `POST`, `PUT`, `DELETE`
- `route`: `/api/sessions`, `/api/sessions/:id/upload`, etc.
- `status`: `2xx`, `3xx`, `4xx`, `5xx` (grouped)

#### Database Metrics

| Metric Name | Type | Labels | Description |
|-------------|------|--------|-------------|
| `db_connections_active` | Gauge | - | Active connections in pool |
| `db_connections_idle` | Gauge | - | Idle connections in pool |
| `db_query_duration_ms` | Histogram | `operation` | Database query latency |
| `db_errors_total` | Counter | `error_type` | Database errors |

**`error_type` Values**:
- `connection_timeout`
- `query_timeout`
- `constraint_violation`
- `unknown`

#### System Metrics

| Metric Name | Type | Labels | Description |
|-------------|------|--------|-------------|
| `process_cpu_usage_percent` | Gauge | - | CPU usage |
| `process_memory_bytes` | Gauge | - | Memory usage |
| `process_uptime_seconds` | Counter | - | Service uptime |

## Logs

### Log Levels

| Level | Usage | Examples |
|-------|-------|----------|
| `ERROR` | Unrecoverable errors, failures | File parsing error, database connection lost |
| `WARN` | Recoverable issues, degraded state | Rate limit approaching, slow query detected |
| `INFO` | Important events | Server started, quote generated, session cleaned up |
| `DEBUG` | Detailed diagnostic info | Request parameters, intermediate calculations |
| `TRACE` | Very detailed tracing | Function entry/exit, variable values |

### Structured Log Fields

All logs include these fields:

| Field | Type | Example | Description |
|-------|------|---------|-------------|
| `timestamp` | ISO 8601 | `2025-12-24T10:30:45.123Z` | When event occurred (UTC) |
| `level` | string | `INFO` | Log level |
| `message` | string | `Quote generated successfully` | Human-readable message |
| `trace_id` | string (UUID) | `a1b2c3d4-e5f6-7890-abcd-ef1234567890` | OpenTelemetry trace ID (if in request context) |
| `span_id` | string | `1234abcd` | Current span ID (if in request context) |
| `service.name` | string | `quote-service` | Service identifier |
| `target` | string | `quote_service::api::routes` | Rust module path |

### Example Logs

#### Successful Quote Generation

```json
{
  "timestamp": "2025-12-24T10:30:45.123Z",
  "level": "INFO",
  "message": "Quote generated successfully",
  "trace_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
  "span_id": "1234abcd",
  "service.name": "quote-service",
  "target": "quote_service::business::pricing",
  "fields": {
    "session_id": "01HX123456789ABCDEFGHJ",
    "model_count": 3,
    "total_price": 45.75,
    "material_id": "pla_standard"
  }
}
```

#### File Upload Error

```json
{
  "timestamp": "2025-12-24T10:31:12.456Z",
  "level": "ERROR",
  "message": "Failed to parse STL file",
  "trace_id": "b2c3d4e5-f6a7-8901-bcde-f12345678901",
  "span_id": "5678efgh",
  "service.name": "quote-service",
  "target": "quote_service::business::file_processor",
  "fields": {
    "error": "Invalid binary STL header",
    "file_name": "corrupted.stl",
    "file_size_bytes": 2048,
    "session_id": "01HX123456789ABCDEFGHJ"
  }
}
```

#### Slow Query Warning

```json
{
  "timestamp": "2025-12-24T10:32:05.789Z",
  "level": "WARN",
  "message": "Slow database query detected",
  "trace_id": "c3d4e5f6-a7b8-9012-cdef-123456789012",
  "span_id": "9012ijkl",
  "service.name": "quote-service",
  "target": "quote_service::persistence",
  "fields": {
    "query_duration_ms": 1250,
    "threshold_ms": 1000,
    "query": "SELECT * FROM materials WHERE active = true"
  }
}
```

## Sensitive Data Handling

**CRITICAL**: The following data MUST NEVER appear in logs or trace attributes:

- `ADMIN_TOKEN` - Admin authentication token
- `MCP_TOKEN` - MCP API token
- `DATABASE_URL` - Database connection string (contains password)
- User email addresses (if feature added in future)
- Payment information (if feature added in future)

**Sanitization Rules**:

1. **Database Queries**: Use `db.statement` with placeholders (`$1`, `$2`) instead of actual values
   ```
   ✅ SELECT * FROM materials WHERE id = $1
   ❌ SELECT * FROM materials WHERE id = 'pla_standard'
   ```

2. **Error Messages**: Strip sensitive data from errors before logging
   ```
   ✅ "Database connection failed"
   ❌ "Connection failed: postgres://user:password@localhost/db"
   ```

3. **HTTP Headers**: Redact `Authorization` header values
   ```
   ✅ http.headers.authorization: "[REDACTED]"
   ❌ http.headers.authorization: "Bearer secret-token-123"
   ```

## Sampling Strategy

**Current**: 100% sampling (all traces exported)

**Future Considerations** (when traffic > 1000 req/min):
- Tail-based sampling: Always sample errors, sample 10% of successful requests
- Head-based sampling: Sample based on trace ID hash
- Priority sampling: Sample admin/MCP requests at higher rate than public requests

## Export Configuration

**Protocol**: OTLP/gRPC
**Endpoint**: Configurable via `OTEL_EXPORTER_OTLP_ENDPOINT` (default: `http://localhost:4317`)
**Batch Export**: Enabled
- Max batch size: 512 spans
- Max queue size: 2048 spans
- Export timeout: 3 seconds
- Schedule delay: 5 seconds

**Retry Policy**:
- Max attempts: 3
- Initial backoff: 1 second
- Max backoff: 5 seconds
- On permanent failure: Drop spans (log warning, don't block application)

## Versioning

This telemetry contract follows semantic versioning:

- **MAJOR**: Breaking changes to span names, metric names, or log structure
- **MINOR**: New spans, metrics, or log fields added
- **PATCH**: Documentation updates, clarifications

**Current Version**: 1.0.0
**Last Updated**: 2025-12-24

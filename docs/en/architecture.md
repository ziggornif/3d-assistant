# 3D Quote Service Architecture

## Overview

The 3D quote service is a full-stack web application built with Rust (backend) and vanilla JavaScript (frontend), allowing users to upload 3D models and get instant quotes for 3D printing.

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                         Clients                                  │
├──────────────┬────────────────────┬─────────────────────────────┤
│  Web Browser │   MCP Clients      │   Admin Interface           │
│              │   (AI Models)      │   (/admin.html)             │
└──────┬───────┴────────┬───────────┴─────────┬───────────────────┘
       │                │                     │
       │ HTTP           │ HTTP                │ HTTP + Cookie Auth
       │                │ Bearer Token        │
       │                │                     │
┌──────▼────────────────▼─────────────────────▼───────────────────┐
│                    Axum Web Server                               │
│                   (Port 3000, Rust)                              │
├──────────────────────────────────────────────────────────────────┤
│  Middleware Stack:                                               │
│  ├─ Security Headers (CSP, HSTS, etc.)                          │
│  ├─ Rate Limiting (100 req/s global)                            │
│  ├─ CORS                                                         │
│  ├─ Authentication (admin_auth, mcp_auth)                       │
│  └─ Tracing/Logging                                             │
├──────────────────────────────────────────────────────────────────┤
│  Routes:                                                         │
│  ├─ /                  → SSR Pages (index, admin)               │
│  ├─ /api/sessions      → Session Management                     │
│  ├─ /api/materials     → Material Catalog                       │
│  ├─ /api/admin/*       → Admin Endpoints (auth required)        │
│  ├─ /mcp               → MCP Protocol (auth required)           │
│  ├─ /static/*          → Static Assets (JS, CSS)                │
│  └─ /uploads/*         → Uploaded 3D Models                     │
└──────┬────────────────┬──────────────┬────────────┬─────────────┘
       │                │              │            │
       │ HTTP           │ HTTP         │ File I/O   │ SQL
       │ Handlers       │ Handlers     │            │
       │                │              │            │
┌──────▼────────┐ ┌─────▼──────┐ ┌────▼──────┐ ┌──▼──────────────┐
│   API Layer   │ │  MCP Layer │ │  Business │ │  Persistence    │
│               │ │            │ │  Logic    │ │  Layer          │
├───────────────┤ ├────────────┤ ├───────────┤ ├─────────────────┤
│ • upload.rs   │ │quote_tools │ │• pricing  │ │ • sessions.rs   │
│ • quote.rs    │ │   .rs      │ │• file_    │ │ • materials.rs  │
│ • materials.rs│ │            │ │  processor│ │ • models.rs     │
│ • admin.rs    │ │            │ │• templates│ │ • quotes.rs     │
│ • ssr.rs      │ │            │ │           │ │                 │
└───────┬───────┘ └─────┬──────┘ └─────┬─────┘ └────────┬────────┘
        │               │              │                │
        └───────────────┴──────────────┴────────────────┘
                              │
                              │ SQL (sqlx)
                              │
                    ┌─────────▼──────────┐
                    │   PostgreSQL DB    │
                    │                    │
                    │  Tables:           │
                    │  • quote_sessions  │
                    │  • uploaded_models │
                    │  • materials       │
                    │  • quotes          │
                    │  • pricing_history │
                    └────────────────────┘
```

## Application Layers

### 1. Presentation Layer

**Location**: `static/js/`, `templates/`

- **Web Components** (Custom Elements):
  - `file-uploader` - 3D file upload with drag & drop
  - `model-viewer` - Interactive 3D visualization (Three.js)
  - `material-selector` - Material selection
  - `quote-summary` - Quote display

- **SSR (Server-Side Rendering)**:
  - Tera templates for server-side rendering
  - Pages: index.html, admin.html

### 2. API Layer

**Location**: `src/api/`

- **Handlers** (`src/api/handlers/`):
  - `upload.rs` - 3D model upload, session management
  - `quote.rs` - Quote generation and retrieval
  - `materials.rs` - Material catalog
  - `admin.rs` - Admin management (materials, pricing, cleanup)
  - `ssr.rs` - HTML page rendering

- **Middleware** (`src/api/middleware/`):
  - `auth.rs` - Admin & MCP authentication
  - `rate_limit.rs` - Rate limiting
  - `sanitize.rs` - Input validation and sanitization
  - `security_headers.rs` - Security headers
  - `error.rs` - Centralized error handling

- **Routes** (`src/api/routes.rs`):
  - Endpoint configuration and middleware application

### 3. MCP Layer (Model Context Protocol)

**Location**: `src/mcp/`

Programmatic interface allowing AI models to interact with the service.

- **MCP Tools**:
  - `list_materials` - List available materials
  - `upload_model` - Upload models in base64
  - `configure_model` - Configure material/quantity
  - `generate_quote` - Generate quote

- **Transport**: StreamableHTTP (rmcp library)

### 4. Business Layer

**Location**: `src/business/`

- **file_processor.rs**:
  - File validation (STL, 3MF)
  - Metadata extraction (volume, dimensions)
  - Zip bomb protection for 3MF

- **pricing.rs**:
  - Price calculation per model
  - Quote generation with fees
  - Minimum order application (€10)

- **templates.rs**:
  - Tera template engine management
  - SSR page rendering

- **session.rs**:
  - Session lifecycle management
  - Expired session cleanup

### 5. Persistence Layer

**Location**: `src/persistence/`

Database abstraction using sqlx with prepared statements.

- **sessions.rs** - CRUD sessions
- **models.rs** - CRUD uploaded models
- **materials.rs** - CRUD materials + pricing history
- **quotes.rs** - CRUD quotes

### 6. Database Layer

**Location**: `src/db/`

- **Embedded Migrations** (`src/db/migrations/`):
  - Automatically executed on startup
  - Incremental migration with versioning

- **Connection**: PostgreSQL connection pool (sqlx)

## Data Models

### quote_sessions
```sql
id               VARCHAR(26) PRIMARY KEY (ULID)
expires_at       TIMESTAMP
created_at       TIMESTAMP
```

### uploaded_models
```sql
id               VARCHAR(26) PRIMARY KEY (ULID)
session_id       VARCHAR(26) REFERENCES quote_sessions
filename         VARCHAR(255)
file_format      VARCHAR(10)  -- 'stl' or '3mf'
file_size_bytes  BIGINT
volume_cm3       DOUBLE PRECISION
dimensions_mm    JSONB        -- {x, y, z}
triangle_count   INT
material_id      VARCHAR(50) REFERENCES materials
file_path        TEXT
preview_url      TEXT
created_at       TIMESTAMP
support_analysis JSONB
```

### materials
```sql
id                  VARCHAR(50) PRIMARY KEY
service_type_id     VARCHAR(50)
name                VARCHAR(100)
description         TEXT
price_per_cm3       DOUBLE PRECISION
color               VARCHAR(7)
properties          JSONB
active              BOOLEAN
created_at          TIMESTAMP
updated_at          TIMESTAMP
```

### quotes
```sql
id              VARCHAR(26) PRIMARY KEY (ULID)
session_id      VARCHAR(26) REFERENCES quote_sessions
total_price     DOUBLE PRECISION
breakdown       JSONB
status          VARCHAR(20)  -- 'pending', 'accepted', 'rejected'
created_at      TIMESTAMP
```

### pricing_history
```sql
id              SERIAL PRIMARY KEY
material_id     VARCHAR(50) REFERENCES materials
old_price       DOUBLE PRECISION
new_price       DOUBLE PRECISION
changed_by      VARCHAR(100)
changed_at      TIMESTAMP
```

## Main Data Flows

### 1. Quote Generation (User Flow)

```
User Browser
    │
    ├─→ POST /api/sessions
    │       │
    │       └─→ SessionService.create_session()
    │               │
    │               └─→ DB INSERT quote_sessions
    │
    ├─→ POST /api/sessions/{id}/models (multipart/form-data)
    │       │
    │       ├─→ validate_file() → file_processor
    │       │       │
    │       │       └─→ process_stl_file() or process_3mf_file()
    │       │               │
    │       │               └─→ Extract volume, dimensions, triangle count
    │       │
    │       └─→ DB INSERT uploaded_models
    │
    ├─→ PATCH /api/sessions/{id}/models/{model_id}
    │       │   Body: {"material_id": "pla", "quantity": 1}
    │       │
    │       └─→ DB UPDATE uploaded_models SET material_id
    │
    └─→ POST /api/sessions/{id}/quote
            │
            ├─→ DB SELECT models WHERE session_id
            │
            ├─→ pricing::calculate_model_price() for each model
            │       │
            │       └─→ price = volume_cm3 × material.price_per_cm3
            │
            ├─→ pricing::generate_quote_breakdown()
            │       │
            │       └─→ total = max(subtotal + fees, MINIMUM_ORDER)
            │
            └─→ DB INSERT quotes
```

### 2. MCP Integration (AI Flow)

```
AI Client
    │
    ├─→ POST /mcp (Bearer: MCP_TOKEN)
    │       │
    │       ├─→ mcp_auth middleware
    │       │
    │       └─→ StreamableHttpService
    │               │
    │               ├─→ list_materials tool
    │               │       │
    │               │       └─→ DB SELECT materials WHERE active=true
    │               │
    │               ├─→ upload_model tool
    │               │       │
    │               │       ├─→ Base64 decode
    │               │       ├─→ validate_file()
    │               │       └─→ DB INSERT uploaded_models
    │               │
    │               ├─→ configure_model tool
    │               │       │
    │               │       └─→ DB UPDATE uploaded_models
    │               │
    │               └─→ generate_quote tool
    │                       │
    │                       ├─→ calculate_model_price()
    │                       └─→ DB INSERT quotes
    │
    └─→ Response (JSON-RPC 2.0)
```

### 3. Administration (Admin Flow)

```
Admin Browser
    │
    ├─→ GET /admin.html
    │       │
    │       └─→ SSR: tera.render("admin.html")
    │
    ├─→ POST /api/admin/login
    │       │   Body: {"token": "..."}
    │       │
    │       ├─→ admin_auth middleware (cookie)
    │       │
    │       └─→ Set-Cookie: admin_token=...
    │
    ├─→ GET /api/admin/materials
    │       │
    │       ├─→ admin_auth middleware
    │       │
    │       └─→ DB SELECT * FROM materials
    │
    ├─→ PATCH /api/admin/materials/{id}
    │       │   Body: {"price_per_cm3": 0.15}
    │       │
    │       ├─→ admin_auth middleware
    │       │
    │       ├─→ DB UPDATE materials
    │       │
    │       └─→ DB INSERT pricing_history
    │
    └─→ POST /api/admin/cleanup
            │
            ├─→ SessionService.cleanup_expired()
            │       │
            │       ├─→ DB DELETE FROM quote_sessions WHERE expires_at < NOW()
            │       │
            │       └─→ fs::remove_dir_all(uploads/{session_id}/)
            │
            └─→ Return CleanupResult
```

## Security

### Authentication

- **Admin**: Cookie-based with secret token
- **MCP**: Bearer token (Authorization header)
- **Public**: No auth required for public endpoints

### Input Protection

- **Sanitization**: All filenames and user inputs
- **Validation**:
  - File format (magic numbers)
  - File size (MAX_FILE_SIZE_MB)
  - Zip bomb protection (3MF)
  - SQL injection: Prepared statements (sqlx)

### Security Headers

- Content-Security-Policy
- X-Frame-Options
- X-Content-Type-Options
- Strict-Transport-Security
- Referrer-Policy

### Rate Limiting

- Global: 100 req/s, burst 500
- Login: 5 req/s, burst 20
- Upload: 10 req/s, burst 50

## Performance

### Optimizations

- **Connection pooling**: PostgreSQL pool (max 5 connections)
- **File processing**: Streaming for large files
- **Static assets**: Static file server
- **SSR**: Compiled templates (Tera)

### Points to Monitor

- N+1 queries in quote generation (one query per model)
- Synchronous file I/O (should be tokio::fs)
- No cache for materials
- No CDN for static assets

## Observability

- **Logging**: `tracing` crate with configurable levels
- **Metrics**: None (TODO: Prometheus)
- **Distributed tracing**: None (TODO: OpenTelemetry)

## Deployment

### Environments

- **Development**: `cargo run` with RUST_LOG=debug
- **Production**: Docker Compose or release build

### System Requirements

- Rust 1.75+
- PostgreSQL 14+
- 512 MB RAM minimum
- 1 GB storage for uploads

## Future Improvements

- JWT for admin authentication
- Webhook system for quote export
- PDF quote export
- Multi-language (i18n)
- Redis cache for materials
- Asynchronous file I/O (tokio::fs)
- Prometheus metrics
- OpenTelemetry tracing

# Implementation Plan: 3D Printing Quote Service

**Branch**: `001-3d-print-quoting` | **Date**: 2025-11-15 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/001-3d-print-quoting/spec.md`

> **Note (v2.0)**: La structure du projet a été simplifiée depuis la rédaction de ce plan. Les répertoires `backend/` et `frontend/` ont été consolidés à la racine (`src/`, `static/`, `templates/`). Les chemins mentionnés ci-dessous reflètent le plan initial.

## Summary

Web application enabling users to upload STL/3MF files, visualize them in 3D, configure printing materials, and receive instant price quotes. Backend built with Rust/Axum for performance and reliability, frontend using vanilla HTML/JS/CSS with native Web Components for reusability. Architecture designed for extensibility to support future services (laser cutting, engraving).

## Technical Context

**Language/Version**: Rust 1.75+ (backend), ES2022 JavaScript (frontend)
**Primary Dependencies**: Axum (web framework), SQLx (database), Three.js (3D rendering), serde (serialization)
**Storage**: SQLite (prototype) → PostgreSQL (production)
**Testing**: cargo test (backend), Jest + Testing Library (frontend), RGAA compliance testing
**Target Platform**: Linux server (backend), Modern browsers with WebGL support (frontend)
**Project Type**: Web application (frontend + backend)
**Performance Goals**: 100 concurrent users, <2s price calculation, <30s file upload/visualization for 10MB files
**Constraints**: p95 < 200ms API response, 50MB max file size, RGAA accessibility compliance
**Scale/Scope**: 100 concurrent users, session-based storage, single-page application

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### I. Code Quality First

- [x] **Static Analysis**: Rust (clippy + rustfmt), JavaScript (ESLint + Prettier) configured
- [x] **Type Safety**: Rust provides compile-time safety, TypeScript definitions for JS APIs
- [x] **Code Reviews**: PR process defined in constitution
- [x] **Documentation**: Rust doc comments, JSDoc for frontend, OpenAPI for contracts
- [x] **Single Responsibility**: Modular architecture with clear separation of concerns
- [x] **No Magic Numbers**: Configuration-driven pricing, named constants for limits

### II. Testing Standards (NON-NEGOTIABLE)

- [x] **Coverage Threshold**: 80% minimum target, 95% for pricing logic (critical path)
- [x] **Test Types Required**:
  - Unit tests: Business logic, pricing calculations
  - Integration tests: API endpoints, file processing pipeline
  - Contract tests: OpenAPI schema validation
- [x] **TDD Approach**: Tests written before implementation for core features
- [x] **Test Naming**: Rust convention `test_module_scenario_outcome`
- [x] **No Mocking Abuse**: Mock only external file system, not internal services
- [x] **CI/CD Gate**: All tests must pass before merge

### III. User Experience Consistency

- [x] **Response Time**: <200ms feedback, progress indicators for uploads
- [x] **Error Messages**: User-friendly with clear actions, logged for debugging
- [x] **Accessibility**: RGAA compliance required (French accessibility standard)
- [x] **Consistent Patterns**: Vanilla JS with Web Components for UI consistency
- [x] **State Management**: Session-based, no unexpected state loss
- [x] **Graceful Degradation**: Handle file processing failures without breaking app

### IV. Performance Requirements

- [x] **Response Time Targets**: p95 < 200ms reads, p95 < 500ms writes (via Axum)
- [x] **Resource Constraints**: 50MB file limit, efficient memory handling with Rust
- [x] **Monitoring**: Structured logging with tracing crate
- [x] **Load Testing**: Required for 100 concurrent user support
- [x] **Performance Budgets**: Bundle size limits for frontend, query optimization for backend

**Constitution Gate Status**: PASSED - All principles satisfied

## Project Structure

### Documentation (this feature)

```text
specs/001-3d-print-quoting/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output (OpenAPI specs)
└── tasks.md             # Phase 2 output (/speckit.tasks command)
```

### Source Code (repository root)

```text
backend/
├── src/
│   ├── main.rs              # Application entry point
│   ├── config.rs            # Configuration management
│   ├── models/
│   │   ├── mod.rs
│   │   ├── material.rs      # Material entity
│   │   ├── quote.rs         # Quote and session entities
│   │   └── service_type.rs  # Service type (3D print, laser, etc.)
│   ├── services/
│   │   ├── mod.rs
│   │   ├── file_processor.rs    # STL/3MF parsing and validation
│   │   ├── pricing.rs           # Quote calculation engine
│   │   └── session.rs           # Session management
│   ├── api/
│   │   ├── mod.rs
│   │   ├── routes.rs            # Route definitions
│   │   ├── handlers/
│   │   │   ├── mod.rs
│   │   │   ├── upload.rs        # File upload endpoints
│   │   │   ├── quote.rs         # Quote generation endpoints
│   │   │   └── admin.rs         # Admin pricing management
│   │   └── middleware/
│   │       ├── mod.rs
│   │       └── error.rs         # Error handling middleware
│   └── db/
│       ├── mod.rs
│       ├── schema.rs            # Database schema
│       └── migrations/          # SQLx migrations
├── tests/
│   ├── contract/               # API contract tests
│   ├── integration/            # End-to-end API tests
│   └── unit/                   # Unit tests for services
├── Cargo.toml
└── Cargo.lock

frontend/
├── index.html                  # Main entry point
├── css/
│   ├── main.css                # Global styles
│   ├── components/             # Component-specific styles
│   └── accessibility.css       # RGAA compliance styles
├── js/
│   ├── main.js                 # Application bootstrap
│   ├── components/
│   │   ├── file-uploader.js    # Web Component: drag-drop uploader
│   │   ├── model-viewer.js     # Web Component: 3D preview (Three.js)
│   │   ├── material-selector.js # Web Component: material options
│   │   └── quote-summary.js    # Web Component: price breakdown
│   ├── services/
│   │   ├── api-client.js       # Backend API communication
│   │   ├── file-parser.js      # Client-side file validation
│   │   └── session-manager.js  # Session state management
│   └── utils/
│       ├── accessibility.js    # RGAA helpers
│       └── formatters.js       # Currency, dimensions formatting
├── assets/
│   └── icons/                  # SVG icons for UI
└── tests/
    ├── unit/                   # Component unit tests
    └── e2e/                    # End-to-end browser tests
```

**Structure Decision**: Web application structure selected due to clear frontend/backend separation. Backend in Rust/Axum handles file processing and pricing logic, frontend in vanilla JS with Web Components provides accessible, framework-free user interface.

## Complexity Tracking

No violations to justify - architecture aligns with constitution principles and uses minimal complexity appropriate for requirements.

## Phase 0: Research

### STL/3MF Parsing in Rust

**Findings**:
- **stl_io** crate: Mature, well-maintained library for STL parsing (binary and ASCII formats)
- **3mf-rs**: Limited support, may need custom implementation for 3MF
- **mesh-io**: Alternative supporting multiple formats but less documented

**Recommendation**: Use `stl_io` for STL files. Implement custom 3MF parser using `zip` and `quick-xml` crates (3MF is ZIP archive with XML model definition).

**Volume Calculation**:
- Signed volume method: Sum of signed tetrahedra volumes from mesh triangles
- Formula: V = Σ (v1 · (v2 × v3)) / 6 for each triangle
- Handles both open and closed meshes with appropriate error handling

### Three.js Integration

**Findings**:
- **STLLoader**: Built-in Three.js loader, production-ready
- **3MFLoader**: Available in Three.js examples, requires additional setup
- **OrbitControls**: Standard for rotate/zoom/pan interaction
- **WebGL compatibility**: 98%+ browser support, graceful fallback possible

**Recommendation**: Use Three.js r150+ with STLLoader and 3MFLoader from examples. Implement WebGL detection with fallback message for unsupported browsers.

### Pricing Algorithm Research

**Volume-based pricing model**:
```
Total Price = (Volume_cm³ × Material_Rate_per_cm³) + Base_Fee + Complexity_Multiplier
```

**Complexity factors**:
- Triangle count (mesh density)
- Bounding box ratio (thin/tall objects need supports)
- Surface area to volume ratio

**Recommendation**: Start with simple volume × material rate model. Add complexity multiplier in future iteration based on actual printing data.

### Database Considerations

**SQLite vs PostgreSQL**:
- SQLite: Zero-config, file-based, suitable for prototype and <100 concurrent users
- PostgreSQL: Better concurrency, production-ready, required for scaling

**Migration path**: SQLx supports both with same codebase. Start SQLite, switch via config.

## Phase 1: Design Artifacts

### Data Model

*See [data-model.md](data-model.md) for detailed schema*

```sql
-- Core entities
CREATE TABLE service_types (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    active BOOLEAN DEFAULT true,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE materials (
    id TEXT PRIMARY KEY,
    service_type_id TEXT REFERENCES service_types(id),
    name TEXT NOT NULL,
    description TEXT,
    price_per_cm3 DECIMAL(10,4) NOT NULL,
    color TEXT,
    properties JSONB,
    active BOOLEAN DEFAULT true,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE quote_sessions (
    id TEXT PRIMARY KEY,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMP NOT NULL,
    status TEXT DEFAULT 'active'
);

CREATE TABLE uploaded_models (
    id TEXT PRIMARY KEY,
    session_id TEXT REFERENCES quote_sessions(id) ON DELETE CASCADE,
    filename TEXT NOT NULL,
    file_format TEXT NOT NULL CHECK (file_format IN ('stl', '3mf')),
    file_size_bytes INTEGER NOT NULL,
    volume_cm3 DECIMAL(12,4),
    dimensions_mm JSONB, -- {x: float, y: float, z: float}
    triangle_count INTEGER,
    material_id TEXT REFERENCES materials(id),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE quotes (
    id TEXT PRIMARY KEY,
    session_id TEXT REFERENCES quote_sessions(id),
    total_price DECIMAL(10,2) NOT NULL,
    breakdown JSONB NOT NULL, -- itemized costs
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE pricing_history (
    id TEXT PRIMARY KEY,
    material_id TEXT REFERENCES materials(id),
    old_price DECIMAL(10,4),
    new_price DECIMAL(10,4) NOT NULL,
    changed_by TEXT,
    changed_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

### API Contracts

*See [contracts/openapi.yaml](contracts/openapi.yaml) for full specification*

**Core Endpoints**:

```yaml
# Session Management
POST /api/sessions
  → Creates new quote session
  ← { session_id, expires_at }

# File Upload
POST /api/sessions/{session_id}/models
  → Multipart file upload (STL/3MF)
  ← { model_id, filename, volume_cm3, dimensions_mm, preview_url }

DELETE /api/sessions/{session_id}/models/{model_id}
  → Removes model from session
  ← 204 No Content

# Materials
GET /api/materials?service_type=3d_printing
  ← [{ id, name, description, price_per_cm3, color, properties }]

# Model Configuration
PATCH /api/sessions/{session_id}/models/{model_id}
  → { material_id }
  ← { model_id, material_id, estimated_price }

# Quote Generation
POST /api/sessions/{session_id}/quote
  → Generates final quote
  ← { quote_id, items: [...], subtotal, fees, total, breakdown }

GET /api/sessions/{session_id}/quote
  ← Current quote calculation (real-time)

# Admin Endpoints (authenticated)
GET /api/admin/materials
PUT /api/admin/materials/{id}
POST /api/admin/materials
GET /api/admin/pricing-history
```

**Error Response Format**:
```json
{
  "error": {
    "code": "INVALID_FILE_FORMAT",
    "message": "Seuls les fichiers STL et 3MF sont acceptés",
    "details": { "received": "obj", "accepted": ["stl", "3mf"] }
  }
}
```

### Component Interfaces

**Frontend Web Components**:

```javascript
// <file-uploader>
// Events: file-selected, upload-progress, upload-complete, upload-error
// Properties: session-id, max-size-mb, accepted-formats

// <model-viewer>
// Events: model-loaded, interaction-start
// Properties: model-url, auto-rotate, show-dimensions

// <material-selector>
// Events: material-selected
// Properties: materials (JSON), selected-material-id

// <quote-summary>
// Events: quote-requested
// Properties: items (JSON), show-breakdown
```

## Architecture Decisions

### ADR-001: Rust Backend with Axum

**Context**: Need performant file processing and reliable concurrent handling for 100+ users.

**Decision**: Use Rust with Axum web framework.

**Rationale**:
- Memory safety without garbage collection pauses
- Excellent concurrency model (async/await with Tokio)
- Strong type system prevents runtime errors in pricing logic
- Performance for file parsing (10-100x faster than interpreted languages)
- SQLx provides compile-time SQL verification

**Alternatives Considered**:
- Node.js: Easier hiring, but GC pauses and single-threaded limitations
- Go: Good concurrency, but less type safety for complex domain logic
- Python/FastAPI: Quick development, but performance concerns for file processing

**Consequences**:
- Steeper learning curve for team
- Longer initial development time
- Better long-term maintainability and performance

### ADR-002: Vanilla JavaScript with Web Components

**Context**: Need accessible, maintainable frontend without framework lock-in.

**Decision**: Use vanilla ES2022 JavaScript with native Web Components.

**Rationale**:
- No framework dependencies to maintain
- Native browser support for components
- Easier RGAA accessibility compliance
- Smaller bundle size
- Long-term browser compatibility

**Alternatives Considered**:
- React: Large ecosystem but heavyweight, accessibility requires extra effort
- Vue: Good DX but adds dependency management overhead
- Svelte: Compiles away but smaller ecosystem

**Consequences**:
- More boilerplate for state management
- Less tooling/component libraries available
- Better performance and accessibility control

### ADR-003: Session-Based Storage (No Auth)

**Context**: MVP focuses on quick quote generation without user accounts.

**Decision**: Anonymous sessions with automatic expiration (24h).

**Rationale**:
- Reduces friction for first-time users
- Simplifies MVP scope
- Aligns with privacy-first approach
- Easy to add authentication layer later

**Consequences**:
- No quote history for users
- Session cleanup required
- Cannot save quotes long-term without authentication

### ADR-004: SQLite for Prototype, PostgreSQL for Production

**Context**: Need database that supports both development simplicity and production scalability.

**Decision**: Use SQLx with SQLite for development/prototype, migrate to PostgreSQL for production.

**Rationale**:
- SQLx abstracts database differences
- SQLite: zero-config local development
- PostgreSQL: production-grade concurrency
- Same codebase, config-driven switch

**Consequences**:
- Must avoid SQLite-specific features
- Testing must cover both databases
- Clear migration path defined

## Risk Assessment

### Technical Risks

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| **3MF parsing complexity** | Medium | High | Start with STL-only MVP, add 3MF in iteration 2. Use well-tested zip/xml crates. |
| **WebGL browser compatibility** | Low | Low | Three.js handles fallbacks. Provide clear browser requirements. |
| **File upload size limits** | Medium | Medium | Client-side validation, chunked upload for large files, clear error messages. |
| **Volume calculation accuracy** | High | Medium | Validate against reference models, unit tests with known volumes, handle non-manifold meshes. |
| **Performance at 100 concurrent users** | High | Medium | Load testing early, horizontal scaling design, SQLite → PostgreSQL migration path. |
| **Memory usage for large files** | Medium | Medium | Stream processing for files, Rust's memory efficiency, 50MB file limit. |

### Business Risks

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| **Inaccurate pricing** | High | Medium | Extensive testing of pricing logic (95% coverage), admin override capability, pricing history audit trail. |
| **User abandonment** | Medium | Medium | <30s upload+render, <3min full flow, progress indicators, clear error recovery. |
| **RGAA non-compliance** | High | Low | Accessibility testing from start, semantic HTML, ARIA labels, keyboard navigation. |

### Contingency Plans

1. **3MF parsing fails**: Release with STL-only support, add 3MF as enhancement
2. **Performance issues**: Scale horizontally, implement caching, optimize hot paths
3. **Pricing disputes**: Maintain audit log, allow admin adjustments, quote versioning

## Implementation Phases

### Phase 1: Foundation (Week 1-2)
**Goal**: Basic infrastructure and file upload

**Deliverables**:
- [ ] Backend project setup (Cargo, Axum, SQLx)
- [ ] Frontend project setup (vanilla JS structure)
- [ ] Database schema migration
- [ ] Session management API
- [ ] STL file upload endpoint
- [ ] Basic file validation
- [ ] File storage (local filesystem)

**Success Criteria**: Can upload STL file and receive session ID

### Phase 2: 3D Visualization (Week 3)
**Goal**: Render uploaded models in browser

**Deliverables**:
- [ ] STL parser integration (volume, dimensions)
- [ ] Three.js setup and STLLoader
- [ ] `<model-viewer>` Web Component
- [ ] Interactive controls (rotate, zoom, pan)
- [ ] Dimension display overlay
- [ ] File format error handling

**Success Criteria**: Upload STL → see interactive 3D preview with dimensions

### Phase 3: Material Configuration (Week 4)
**Goal**: Material selection and assignment

**Deliverables**:
- [ ] Materials API endpoint
- [ ] Material database seeding
- [ ] `<material-selector>` Web Component
- [ ] Model-material assignment API
- [ ] Real-time price estimate display
- [ ] Multiple model support

**Success Criteria**: Upload multiple files, assign different materials

### Phase 4: Pricing Engine (Week 5)
**Goal**: Accurate quote calculation

**Deliverables**:
- [ ] Pricing calculation service (95% test coverage)
- [ ] Quote generation API
- [ ] `<quote-summary>` Web Component
- [ ] Itemized breakdown display
- [ ] Price update on configuration change
- [ ] Quote persistence

**Success Criteria**: Full quote flow with accurate pricing

### Phase 5: Admin Interface (Week 6)
**Goal**: Non-technical price management

**Deliverables**:
- [ ] Admin authentication (basic)
- [ ] Material CRUD endpoints
- [ ] Price update interface
- [ ] Pricing history audit log
- [ ] Material activation/deactivation

**Success Criteria**: Admin can update prices, changes reflect immediately

### Phase 6: Polish & Performance (Week 7-8)
**Goal**: Production readiness

**Deliverables**:
- [ ] RGAA accessibility compliance
- [ ] Performance optimization
- [ ] Load testing (100 concurrent users)
- [ ] Error handling refinement
- [ ] 3MF support (if time permits)
- [ ] Documentation
- [ ] Deployment configuration

**Success Criteria**: All acceptance criteria met, performance targets achieved

## Dependencies & Critical Path

```
Phase 1 (Foundation)
    ↓
Phase 2 (Visualization) ←── Three.js expertise required
    ↓
Phase 3 (Materials) ←── Can parallel with Phase 2 backend
    ↓
Phase 4 (Pricing) ←── CRITICAL: Must have 95% test coverage
    ↓
Phase 5 (Admin) ←── Can start after Phase 4 pricing service
    ↓
Phase 6 (Polish)
```

**Critical Path Items**:
1. STL parsing and volume calculation accuracy
2. Pricing engine correctness and test coverage
3. RGAA accessibility compliance
4. Performance at scale

## Quickstart Guide

*See [quickstart.md](quickstart.md) for developer setup*

```bash
# Clone and setup
git clone <repo>
cd 3d-assistant
git checkout 001-3d-print-quoting

# Backend
cd backend
cargo build
cp .env.example .env
cargo sqlx database setup
cargo run

# Frontend (separate terminal)
cd frontend
npx serve .  # or python -m http.server

# Test
cargo test
npm test  # frontend tests
```

## Next Steps

1. ✅ Complete Phase 0 Research
2. ✅ Complete Phase 1 Design Artifacts
3. → Generate detailed tasks with `/speckit.tasks`
4. → Begin Phase 1 implementation
5. → Create research.md, data-model.md, quickstart.md, contracts/ as separate files

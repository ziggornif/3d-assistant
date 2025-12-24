# Technical Backlog: 3D Printing Quote Service

**Feature Branch**: `001-3d-print-quoting`
**Last Updated**: 2025-11-15

> **Note (v2.0)**: La structure du projet a été simplifiée. Les chemins `frontend/` et `backend/` ont été consolidés à la racine (`src/`, `static/`, `templates/`). Les références ci-dessous reflètent la structure initiale.

## Identified Improvements

### High Priority

#### BL-001: Model 3D Positioning on Grid ✅ COMPLETED
**Category**: UX/Visual
**Identified**: 2025-11-15 during testing
**Completed**: 2025-11-15
**Component**: `frontend/js/components/model-viewer.js`

**Implementation**:
- Added rotation from Z-up (slicer) to Y-up (Three.js) coordinate system
- Used `geometry.center()` to center model at origin
- Computed bounding box and translated Y so bottom touches grid (Y=0)
- Dynamically scaled GridHelper based on model footprint
- Improved camera positioning for better viewing angle

---

### Medium Priority

#### BL-002: UUID to ULID Migration ✅ COMPLETED
**Category**: Infrastructure/Database
**Identified**: 2025-11-15 during code review
**Completed**: 2025-11-15
**Components**: Backend models, database schema

**Implementation**:
- Added `ulid = "1.1"` crate dependency
- Replaced all `Uuid::new_v4()` with `Ulid::new()` in models and handlers
- Removed `uuid` crate dependency entirely
- Backward compatible with existing VARCHAR IDs in database
- New IDs are lexicographically sortable (e.g., `01JCV8E3X1MFXZ6BNQWRGY9P0D`)

---

## Critical Issues

#### BL-007: MCP Security Vulnerabilities ⚠️ CRITICAL
**Category**: Security/Stability
**Identified**: 2025-12-23 during PR #5 code review
**Priority**: P0 - BLOCKING
**Status**: Planned (US 5.1)
**Components**: `src/mcp/quote_tools.rs`, `src/api/routes.rs`, `tests/mcp_integration_test.rs`

**Issues Identified**:
1. **No Authentication on MCP Endpoint** (CRITICAL)
   - `/mcp` endpoint has no authentication middleware
   - Anyone can upload files and generate quotes
   - Risk: Resource exhaustion, abuse

2. **Unsafe Unwrap** (HIGH)
   - Line 365: `model.material_id.as_ref().unwrap()`
   - Can panic if model missing material_id
   - Risk: Service crashes

3. **Price Manipulation** (HIGH - Financial Risk)
   - Lines 311, 373: Price defaults to 0.0 on parse errors
   - `unit_price.to_string().parse::<f64>().unwrap_or(0.0)`
   - Risk: Free quotes, revenue loss

4. **Test Failures** (HIGH)
   - `test_session_cleanup` consistently fails
   - `assert!(true)` at line 221 - useless assertion
   - Risk: CI unreliability

5. **Quantity Parameter Inconsistency** (MEDIUM)
   - Parameter accepted but hardcoded to 1 in quote generation
   - Risk: User confusion, incorrect quotes

**Required Actions** (See US 5.1 in spec.md):
- [ ] Add MCP authentication middleware
- [ ] Replace unwrap with proper error handling
- [ ] Fix price calculation error handling
- [ ] Fix or remove failing tests
- [ ] Fix quantity persistence or remove parameter
- [ ] Add error handling integration tests
- [ ] Update MCP documentation

**References**:
- PR #5: https://github.com/ziggornif/3d-assistant/pull/5
- Code Review: Internal review 2025-12-23
- User Story: US 5.1 in spec.md

---

## Future Enhancements

#### BL-003: 3MF File Format Support
**Category**: Feature
**Status**: Planned (Phase 7, T126-T127)

Currently only STL files are fully supported. 3MF support is planned for:
- Backend: Parse 3MF with proper library
- Frontend: 3MFLoader integration in Three.js

---

#### BL-004: WebGL Fallback ✅ COMPLETED
**Category**: Resilience
**Completed**: 2025-11-15

**Implementation**:
- Added graceful fallback when WebGL is not available
- Displays user-friendly message with package icon
- Shows model dimensions after upload (X, Y, Z)
- Clean UI with styled container and dimension display
- Emits viewer-ready event with fallbackMode flag for parent integration

---

#### BL-005: Session Cleanup with File Deletion ✅ COMPLETED
**Category**: Infrastructure/Maintenance
**Identified**: 2025-11-15 during backlog review
**Completed**: 2025-11-15
**Components**: `backend/src/services/session.rs`, `backend/src/api/handlers/admin.rs`

**Implementation**:
- Enhanced SessionService to track upload directory path
- `cleanup_expired()` method now:
  - Fetches expired session IDs from database
  - Deletes upload directories (`uploads/{session_id}/`) for each session
  - Cascades deletion to uploaded_models and quotes tables
  - Returns detailed CleanupResult with stats and errors
- Added admin endpoint `POST /api/admin/cleanup` to trigger cleanup
- CleanupResult includes sessions_deleted, directories_deleted, and error list
- Full transaction safety with error handling per directory

---

#### BL-006: Rust Compiler Warnings Cleanup ✅ COMPLETED
**Category**: Code Quality
**Completed**: 2025-11-15

**Implementation**:
- Added `#[allow(dead_code)]` to planned but unused structs/methods
- Added `#[allow(unused_imports)]` to re-exports for future use
- Eliminated all 25+ dead_code/unused warnings
- Code now compiles with zero warnings (except sqlx-postgres external)

---

## Bugs Fixed During Testing

### BF-001: Multipart Upload Body Limit (RESOLVED)
**Issue**: Browser uploads failed with "failed to read stream"
**Root Cause**: Axum default body limit of 2MB
**Fix**: Added `DefaultBodyLimit::max(100MB)` in routes.rs

### BF-002: UUID Mismatch in File Storage (RESOLVED)
**Issue**: 404 errors when fetching uploaded STL files
**Root Cause**: File saved with different UUID than model.id
**Fix**: Create UploadedModel first, use model.id for filename

### BF-003: Three.js MIME Type Blocking (RESOLVED)
**Issue**: CDN returning wrong MIME types, browser blocking scripts
**Root Cause**: Legacy JS bundles incompatible with modern browsers
**Fix**: Migrated to ES modules with importmap

---

## Performance Considerations

- Large STL files (>20MB) may cause browser lag during parsing
- Consider web worker for Three.js geometry processing
- Session cleanup available via admin endpoint (consider cron job for automation)
- ✅ File storage cleanup implemented with session cleanup

---

## Notes

- All items should be reviewed before production deployment
- High priority items affect user experience directly
- Medium priority items improve maintainability and performance
- Track changes in git with proper commit messages

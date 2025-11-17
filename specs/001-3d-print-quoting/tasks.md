# Tasks: 3D Printing Quote Service

**Input**: Design documents from `/specs/001-3d-print-quoting/`
**Prerequisites**: plan.md (required), spec.md (required for user stories)

**Tests**: Included as TDD approach is specified in constitution (tests written before implementation for core features).

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3, US4)
- Include exact file paths in descriptions

## Path Conventions

> **Note (v2.0)**: La structure du projet a été simplifiée. Les répertoires `backend/` et `frontend/` ont été consolidés à la racine :
> - `backend/src/` → `src/`
> - `backend/Cargo.toml` → `Cargo.toml`
> - `frontend/js/` → `static/js/`
> - `frontend/css/` → `static/css/`
> - `frontend/*.html` → `templates/` (SSR avec Tera)
>
> Les chemins ci-dessous reflètent le plan initial et sont conservés pour référence historique.

- **Web app**: `backend/src/`, `frontend/js/`, `frontend/css/`
- Rust backend with Axum, vanilla JavaScript frontend with Web Components

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and basic structure

- [x] T001 Create project directory structure: backend/, frontend/, per implementation plan
- [x] T002 Initialize Rust project with Cargo.toml in backend/
- [x] T003 [P] Add Axum, SQLx, serde, tokio dependencies to backend/Cargo.toml
- [x] T004 [P] Configure clippy and rustfmt in backend/.rustfmt.toml and backend/clippy.toml
- [x] T005 [P] Create frontend directory structure: js/, css/, assets/, tests/ in frontend/
- [x] T006 [P] Configure ESLint and Prettier for frontend in frontend/.eslintrc.json
- [x] T007 [P] Create backend/.env.example with DATABASE_URL, PORT, and environment variables
- [x] T008 Setup Jest configuration for frontend tests in frontend/jest.config.js

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [x] T009 Create database migrations directory in backend/src/db/migrations/
- [x] T010 Create service_types table migration in backend/src/db/migrations/001_service_types.sql
- [x] T011 Create materials table migration in backend/src/db/migrations/002_materials.sql
- [x] T012 Create quote_sessions table migration in backend/src/db/migrations/003_quote_sessions.sql
- [x] T013 Create uploaded_models table migration in backend/src/db/migrations/004_uploaded_models.sql
- [x] T014 Create quotes table migration in backend/src/db/migrations/005_quotes.sql
- [x] T015 Create pricing_history table migration in backend/src/db/migrations/006_pricing_history.sql
- [x] T016 Implement database connection pool in backend/src/db/mod.rs
- [x] T017 [P] Create ServiceType model in backend/src/models/service_type.rs
- [x] T018 [P] Create Material model in backend/src/models/material.rs
- [x] T019 [P] Create QuoteSession model in backend/src/models/quote.rs
- [x] T020 Implement Axum router setup in backend/src/api/routes.rs
- [x] T021 [P] Implement error handling middleware in backend/src/api/middleware/error.rs
- [x] T022 [P] Implement structured logging with tracing in backend/src/main.rs
- [x] T023 Create configuration management in backend/src/config.rs
- [x] T024 Implement session management service in backend/src/services/session.rs
- [x] T025 Create main entry point with Axum server in backend/src/main.rs
- [x] T026 Seed initial service types (3D printing) in backend/src/db/seed.sql
- [x] T027 Seed initial materials (PLA, ABS, PETG, Resin) in backend/src/db/seed.sql
- [x] T028 [P] Create frontend index.html with basic structure in frontend/index.html
- [x] T029 [P] Create main.css with global styles in frontend/css/main.css
- [x] T030 [P] Create accessibility.css for RGAA compliance in frontend/css/accessibility.css
- [x] T031 Create API client service in frontend/js/services/api-client.js
- [x] T032 Create session manager service in frontend/js/services/session-manager.js

**Checkpoint**: Foundation ready - user story implementation can now begin

---

## Phase 3: User Story 1 - Upload and Visualize 3D Files (Priority: P1) 🎯 MVP

**Goal**: Users can upload STL/3MF files and see interactive 3D previews with dimensions

**Independent Test**: Upload an STL file → see 3D model rendered with rotate/zoom/pan controls and dimensions displayed

### Tests for User Story 1

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [x] T033 [P] [US1] Contract test for POST /api/sessions in tests/api/sessions.spec.js
- [x] T034 [P] [US1] Contract test for POST /api/sessions/{id}/models in tests/api/quote.spec.js
- [x] T035 [P] [US1] Unit test for STL file validation in backend/src/services/file_processor.rs
- [x] T036 [P] [US1] Unit test for volume calculation in backend/src/services/file_processor.rs
- [x] T037 [P] [US1] Unit test for dimension extraction in backend/src/services/file_processor.rs
- [x] T038 [P] [US1] Frontend unit test for file-uploader component in frontend/tests/unit/file-uploader.test.js
- [x] T039 [P] [US1] Frontend unit test for model-viewer component in frontend/tests/unit/file-uploader.test.js

### Implementation for User Story 1

- [x] T040 [P] [US1] Create UploadedModel entity in backend/src/models/quote.rs
- [x] T041 [P] [US1] Add stl_io dependency for STL parsing in backend/Cargo.toml
- [x] T042 [US1] Implement STL file parser in backend/src/services/file_processor.rs
- [x] T043 [US1] Implement volume calculation (signed tetrahedra method) in backend/src/services/file_processor.rs
- [x] T044 [US1] Implement dimension extraction (bounding box) in backend/src/services/file_processor.rs
- [x] T045 [US1] Implement file validation (format, size limit 50MB) in backend/src/services/file_processor.rs
- [x] T046 [US1] Create POST /api/sessions endpoint in backend/src/api/handlers/upload.rs
- [x] T047 [US1] Create POST /api/sessions/{id}/models upload endpoint in backend/src/api/handlers/upload.rs
- [x] T048 [US1] Implement file storage service (local filesystem) in backend/src/services/file_storage.rs
- [x] T049 [US1] Create DELETE /api/sessions/{id}/models/{model_id} endpoint in backend/src/api/handlers/upload.rs
- [x] T050 [US1] Add Three.js library to frontend in frontend/index.html
- [x] T051 [US1] Create file-uploader Web Component with drag-drop in frontend/js/components/file-uploader.js
- [x] T052 [US1] Implement file type validation (client-side) in frontend/js/services/file-parser.js
- [x] T053 [US1] Implement upload progress indicator in frontend/js/components/file-uploader.js
- [x] T054 [US1] Create model-viewer Web Component with Three.js in frontend/js/components/model-viewer.js
- [x] T055 [US1] Implement STLLoader integration in frontend/js/components/model-viewer.js
- [x] T056 [US1] Implement OrbitControls (rotate, zoom, pan) in frontend/js/components/model-viewer.js
- [x] T057 [US1] Display model dimensions overlay in frontend/js/components/model-viewer.js
- [x] T058 [US1] Handle multiple file uploads in frontend/js/main.js
- [x] T059 [US1] Implement error messages for invalid formats in frontend/js/components/file-uploader.js
- [x] T060 [US1] Add ARIA labels for accessibility in frontend/js/components/file-uploader.js
- [x] T061 [US1] Add ARIA labels for model-viewer accessibility in frontend/js/components/model-viewer.js
- [x] T062 [US1] Create file-uploader styles in frontend/css/components/file-uploader.css
- [x] T063 [US1] Create model-viewer styles in frontend/css/components/model-viewer.css

**Checkpoint**: User Story 1 complete - users can upload files and see 3D previews

---

## Phase 4: User Story 2 - Configure Print Options per Model (Priority: P2)

**Goal**: Users can select materials for each uploaded model and see immediate price estimates

**Independent Test**: Upload file → select material from dropdown → see selection confirmed and estimate updated

### Tests for User Story 2

- [x] T064 [P] [US2] Contract test for GET /api/materials in tests/api/materials.spec.js
- [x] T065 [P] [US2] Contract test for PATCH /api/sessions/{id}/models/{id} in tests/api/quote.spec.js
- [x] T066 [P] [US2] Unit test for material assignment in backend/src/services/pricing.rs
- [x] T067 [P] [US2] Frontend unit test for material-selector component in frontend/tests/unit/material-selector.test.js

### Implementation for User Story 2

- [x] T068 [US2] Create GET /api/materials endpoint in backend/src/api/handlers/materials.rs
- [x] T069 [US2] Implement material filtering by service_type in backend/src/api/handlers/materials.rs
- [x] T070 [US2] Create PATCH /api/sessions/{id}/models/{id} endpoint in backend/src/api/handlers/quote.rs
- [x] T071 [US2] Implement material assignment to model in backend/src/services/session.rs
- [x] T072 [US2] Calculate basic price estimate (volume × material rate) in backend/src/services/pricing.rs
- [x] T073 [US2] Create material-selector Web Component in frontend/js/components/material-selector.js
- [x] T074 [US2] Display material options with descriptions in frontend/js/components/material-selector.js
- [x] T075 [US2] Implement material selection event handling in frontend/js/components/material-selector.js
- [x] T076 [US2] Show visual confirmation of selection in frontend/js/components/material-selector.js
- [x] T077 [US2] Display real-time price estimate per model in frontend/js/components/material-selector.js
- [x] T078 [US2] Support independent material selection per model in frontend/js/main.js
- [x] T079 [US2] Add ARIA labels for material-selector accessibility in frontend/js/components/material-selector.js
- [x] T080 [US2] Create material-selector styles in frontend/css/components/material-selector.css
- [x] T081 [US2] Create currency formatter utility in frontend/js/utils/formatters.js

**Checkpoint**: User Story 2 complete - users can configure materials independently per model

---

## Phase 5: User Story 3 - Receive Instant Price Quote (Priority: P3)

**Goal**: Users see detailed itemized price breakdown with automatic updates on configuration changes

**Independent Test**: Configure multiple models → request quote → see itemized breakdown per model and total within 3 seconds

### Tests for User Story 3

- [x] T082 [P] [US3] Contract test for POST /api/sessions/{id}/quote in tests/api/quote.spec.js
- [x] T083 [P] [US3] Contract test for GET /api/sessions/{id}/quote in tests/api/quote.spec.js
- [x] T084 [P] [US3] Unit test for pricing calculation (95% coverage) in backend/src/services/pricing.rs
- [x] T085 [P] [US3] Unit test for itemized breakdown in backend/src/services/pricing.rs
- [x] T086 [P] [US3] Unit test for price accuracy (2 decimal places) in backend/src/services/pricing.rs
- [x] T087 [P] [US3] Frontend unit test for quote-summary component in frontend/tests/unit/quote-summary.test.js

### Implementation for User Story 3

- [x] T088 [US3] Create Quote entity in backend/src/models/quote.rs
- [x] T089 [US3] Implement comprehensive pricing calculation in backend/src/services/pricing.rs
- [x] T090 [US3] Add base fee calculation in backend/src/services/pricing.rs
- [x] T091 [US3] Implement itemized breakdown generation in backend/src/services/pricing.rs
- [x] T092 [US3] Ensure price accuracy to 2 decimal places in backend/src/services/pricing.rs
- [x] T093 [US3] Create POST /api/sessions/{id}/quote endpoint in backend/src/api/handlers/quote.rs
- [x] T094 [US3] Create GET /api/sessions/{id}/quote endpoint in backend/src/api/handlers/quote.rs
- [x] T095 [US3] Persist generated quotes in backend/src/api/handlers/quote.rs
- [x] T096 [US3] Create quote-summary Web Component in frontend/js/components/quote-summary.js
- [x] T097 [US3] Display individual prices per model in frontend/js/components/quote-summary.js
- [x] T098 [US3] Display itemized costs (material, volume, fees) in frontend/js/components/quote-summary.js
- [x] T099 [US3] Calculate and display total sum in frontend/js/components/quote-summary.js
- [x] T100 [US3] Implement automatic price recalculation on changes in frontend/js/main.js
- [x] T101 [US3] Add loading indicator during calculation in frontend/js/components/quote-summary.js
- [x] T102 [US3] Add ARIA labels for quote-summary accessibility in frontend/js/components/quote-summary.js
- [x] T103 [US3] Create quote-summary styles in frontend/css/components/quote-summary.css

**Checkpoint**: User Story 3 complete - full quote flow with accurate pricing

---

## Phase 6: User Story 4 - Administrator Manages Pricing (Priority: P4)

**Goal**: Administrators can update material prices and add new materials without code changes

**Independent Test**: Login as admin → update material price → verify new quotes reflect updated price immediately

### Tests for User Story 4

- [x] T104 [P] [US4] Contract test for GET /api/admin/materials in tests/api/admin.spec.js
- [x] T105 [P] [US4] Contract test for PUT /api/admin/materials/{id} in tests/api/admin.spec.js
- [x] T106 [P] [US4] Contract test for POST /api/admin/materials in tests/api/admin.spec.js
- [x] T107 [P] [US4] Unit test for pricing history tracking in tests/api/admin.spec.js
- [x] T108 [P] [US4] Unit test for admin authentication in backend/src/api/middleware/auth.rs

### Implementation for User Story 4

- [x] T109 [US4] Implement basic admin authentication middleware in backend/src/api/middleware/auth.rs
- [x] T110 [US4] Create GET /api/admin/materials endpoint in backend/src/api/handlers/admin.rs
- [x] T111 [US4] Create PUT /api/admin/materials/{id} endpoint in backend/src/api/handlers/admin.rs
- [x] T112 [US4] Create POST /api/admin/materials endpoint in backend/src/api/handlers/admin.rs
- [x] T113 [US4] Implement pricing history tracking in backend/src/services/pricing.rs
- [x] T114 [US4] Create GET /api/admin/pricing-history endpoint in backend/src/api/handlers/admin.rs
- [x] T115 [US4] Create admin login page in frontend/admin.html
- [x] T116 [US4] Create admin pricing management interface in frontend/js/admin/main.js
- [x] T117 [US4] Display all materials with current prices in frontend/js/admin/main.js
- [x] T118 [US4] Implement price update form in frontend/js/admin/main.js
- [x] T119 [US4] Show confirmation and change history in frontend/js/admin/main.js
- [x] T120 [US4] Implement material activation/deactivation in frontend/js/admin/main.js
- [x] T121 [US4] Create add new material form in frontend/js/admin/main.js
- [x] T122 [US4] Create admin interface styles in frontend/css/admin.css

**Checkpoint**: User Story 4 complete - admin can manage pricing without developer intervention

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

- [x] T123 [P] Complete RGAA accessibility audit in frontend/css/accessibility.css
- [x] T124 [P] Add keyboard navigation support across all components in frontend/js/utils/accessibility.js
- [ ] T125 Implement WebGL detection with fallback message in frontend/js/main.js
- [ ] T126 [P] Add 3MF file support parser in backend/src/services/file_processor.rs
- [ ] T127 [P] Add 3MFLoader integration in frontend/js/components/model-viewer.js
- [ ] T128 Performance optimization for file processing in backend/src/services/file_processor.rs
- [ ] T129 Implement session cleanup cron job in backend/src/main.rs
- [x] T130 Add request rate limiting middleware in backend/src/api/middleware/rate_limit.rs
- [ ] T131 [P] Load testing for 100 concurrent users using k6 or similar
- [x] T132 [P] Create README.md with setup instructions
- [x] T133 [P] Create API documentation from OpenAPI spec in docs/api.yaml
- [x] T134 Security hardening: input sanitization in backend/src/api/middleware/sanitize.rs
- [ ] T135 [P] Add error boundaries and recovery in frontend/js/main.js
- [ ] T136 Implement graceful degradation for file processing failures in backend/
- [ ] T137 Run full integration test suite
- [ ] T138 Verify all success criteria from spec.md

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3-6)**: All depend on Foundational phase completion
  - User stories can proceed in priority order (P1 → P2 → P3 → P4)
  - Or in parallel if team has capacity
- **Polish (Phase 7)**: Depends on at least User Story 3 (P3) being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2) - No dependencies on other stories
- **User Story 2 (P2)**: Can start after Foundational (Phase 2) - Uses models from US1 but independently testable
- **User Story 3 (P3)**: Can start after Foundational (Phase 2) - Uses US2 pricing logic but independently testable
- **User Story 4 (P4)**: Can start after Foundational (Phase 2) - Completely independent

### Within Each User Story

- Tests MUST be written and FAIL before implementation
- Models before services
- Services before endpoints
- Backend before frontend integration
- Core implementation before polish
- Story complete before moving to next priority

### Parallel Opportunities

- All Setup tasks marked [P] can run in parallel (T003-T008)
- Foundational tasks marked [P] can run in parallel (T017-T018, T021-T022, T028-T030)
- All tests for a user story marked [P] can run in parallel
- Model creation within a story marked [P] can run in parallel
- Different user stories can be worked on in parallel by different team members

---

## Parallel Example: User Story 1

```bash
# Launch all tests for User Story 1 together:
Task: "Contract test for POST /api/sessions in backend/tests/contract/test_sessions.rs"
Task: "Contract test for POST /api/sessions/{id}/models in backend/tests/contract/test_upload.rs"
Task: "Unit test for STL file validation in backend/tests/unit/test_file_validator.rs"
Task: "Unit test for volume calculation in backend/tests/unit/test_volume_calculator.rs"
Task: "Unit test for dimension extraction in backend/tests/unit/test_dimensions.rs"
Task: "Frontend unit test for file-uploader component in frontend/tests/unit/test_file_uploader.js"
Task: "Frontend unit test for model-viewer component in frontend/tests/unit/test_model_viewer.js"

# After tests fail, launch independent model/component creation:
Task: "Create UploadedModel entity in backend/src/models/quote.rs"
Task: "Add stl_io dependency for STL parsing in backend/Cargo.toml"
```

---

## Parallel Example: User Story 3 (Pricing Engine)

```bash
# All pricing tests can run in parallel:
Task: "Contract test for POST /api/sessions/{id}/quote in backend/tests/contract/test_quote.rs"
Task: "Contract test for GET /api/sessions/{id}/quote in backend/tests/contract/test_quote.rs"
Task: "Unit test for pricing calculation (95% coverage) in backend/tests/unit/test_pricing.rs"
Task: "Unit test for itemized breakdown in backend/tests/unit/test_pricing.rs"
Task: "Unit test for price accuracy (2 decimal places) in backend/tests/unit/test_pricing.rs"
Task: "Frontend unit test for quote-summary component in frontend/tests/unit/test_quote_summary.js"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001-T008)
2. Complete Phase 2: Foundational (T009-T032) - **CRITICAL**
3. Complete Phase 3: User Story 1 (T033-T063)
4. **STOP and VALIDATE**: Test User Story 1 independently
5. Users can upload and visualize files - core value delivered

### Incremental Delivery

1. Complete Setup + Foundational → Foundation ready
2. Add User Story 1 → Test independently → **MVP: File Upload & Visualization**
3. Add User Story 2 → Test independently → **Enhanced: Material Configuration**
4. Add User Story 3 → Test independently → **Complete: Quote Generation**
5. Add User Story 4 → Test independently → **Admin: Pricing Management**
6. Polish phase → Test all together → **Production Ready**

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together (Week 1-2)
2. Once Foundational is done:
   - Developer A: User Story 1 (P1)
   - Developer B: Can assist with US1 tests or prepare US2 tests
3. After US1 complete:
   - Developer A: User Story 2 (P2)
   - Developer B: User Story 3 (P3)
4. After US2/US3 complete:
   - Both: User Story 4 (P4) and Polish

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Verify tests fail before implementing (TDD approach)
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- 95% test coverage required for pricing logic (US3)
- RGAA accessibility compliance is mandatory
- All API responses must include proper error handling
- Avoid: vague tasks, same file conflicts, cross-story dependencies that break independence

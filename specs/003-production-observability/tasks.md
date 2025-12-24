# Tasks: Production Deployment and Observability

**Input**: Design documents from `/specs/003-production-observability/`
**Prerequisites**: plan.md, spec.md, research.md, contracts/, quickstart.md

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and basic structure

- [x] T001 Create new observability module directory at src/observability/
- [x] T002 [P] Add OpenTelemetry dependencies to Cargo.toml (opentelemetry 0.21+, opentelemetry-otlp, tracing-opentelemetry)
- [x] T003 [P] Create deployment directory structure at deployment/ with scripts/ and nginx/ subdirectories
- [x] T004 [P] Create documentation directories at docs/deployment/ and docs/observability/

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [x] T005 Create observability module scaffold in src/observability/mod.rs with public re-exports
- [x] T006 [P] Create placeholder files: src/observability/tracing.rs, src/observability/metrics.rs, src/observability/logging.rs
- [x] T007 Enhance config.rs to load new environment variables structure (ENVIRONMENT, OTEL_EXPORTER_OTLP_ENDPOINT, OTEL_SERVICE_NAME)
- [x] T008 Add environment variable validation in src/config.rs with clear error messages for missing required vars

**Checkpoint**: Foundation ready - user story implementation can now begin in parallel

---

## Phase 3: User Story 1 - Production Configuration Management (Priority: P1) 🎯 MVP

**Goal**: Enable secure, environment-specific configuration via environment variables for flexible deployment across platforms

**Independent Test**: Deploy to a test environment, set environment variables, verify service starts successfully, confirm different configurations work for dev vs prod modes without code changes

### Implementation for User Story 1

- [x] T009 [US1] Implement ENVIRONMENT-based configuration mode detection in src/config.rs (development vs production)
- [x] T010 [US1] Add configuration validation logic in src/config.rs to fail fast with clear error messages for missing secrets
- [x] T011 [US1] Create .env.example file at repository root documenting all environment variables with required/optional status
- [x] T012 [US1] Implement secrets sanitization in logging configuration to prevent DATABASE_URL, ADMIN_TOKEN, MCP_TOKEN from appearing in logs
- [x] T013 [US1] Update main.rs to initialize configuration with environment-based setup
- [x] T014 [US1] Add integration test in tests/integration/config_test.rs to verify configuration loading and validation
- [x] T015 [US1] Document environment variables in docs/deployment/configuration.md with examples for each platform

**Checkpoint**: Configuration management complete - service can be configured via env vars for any deployment platform

---

## Phase 4: User Story 2 - Application Observability with OpenTelemetry (Priority: P1)

**Goal**: Comprehensive observability instrumentation using OpenTelemetry for traces, metrics, and structured logs

**Independent Test**: Run instrumented application, generate requests (upload files, generate quotes), verify traces/metrics/logs appear in OTLP collector with proper context propagation

### Implementation for User Story 2

#### Tracing

- [ ] T016 [P] [US2] Implement OpenTelemetry tracer initialization in src/observability/tracing.rs with OTLP exporter configuration
- [ ] T017 [P] [US2] Create tracing-opentelemetry bridge layer in src/observability/tracing.rs connecting tracing crate to OpenTelemetry
- [ ] T018 [US2] Add trace context propagation middleware in src/api/middleware/tracing.rs
- [ ] T019 [US2] Instrument file upload operation with custom spans in src/business/file_processor.rs (validate_file, parse_stl_file, calculate_volume)
- [ ] T020 [US2] Instrument quote calculation with custom spans in src/business/pricing.rs (fetch_material, compute_model_price, apply_minimum_order)
- [ ] T021 [US2] Instrument session cleanup with custom spans in src/business/session_cleanup.rs (find_expired_sessions, delete_session_files, delete_session_data)
- [ ] T022 [US2] Add database query instrumentation spans for all sqlx queries

#### Metrics

- [ ] T023 [P] [US2] Implement metrics setup in src/observability/metrics.rs with OTLP metrics exporter
- [ ] T024 [P] [US2] Create business metrics in src/observability/metrics.rs (quotes_generated_total, file_upload_size_bytes, quote_calculation_duration_ms)
- [ ] T025 [P] [US2] Create technical metrics in src/observability/metrics.rs (http_requests_total, http_request_duration_ms, db_connections_active, db_query_duration_ms)
- [ ] T026 [US2] Instrument HTTP request handler in src/api/routes.rs to record request metrics
- [ ] T027 [US2] Add metric recording to quote generation flow in src/business/pricing.rs
- [ ] T028 [US2] Add metric recording to file upload flow in src/business/file_processor.rs
- [ ] T029 [US2] Add database connection pool metrics recording using sqlx pool stats

#### Logging

- [ ] T030 [P] [US2] Implement structured JSON logging configuration in src/observability/logging.rs for production environment
- [ ] T031 [P] [US2] Implement human-readable logging configuration in src/observability/logging.rs for development environment
- [ ] T032 [US2] Configure logging subscriber in main.rs to use environment-dependent format
- [ ] T033 [US2] Add trace context fields (trace_id, span_id) to all log entries
- [ ] T034 [US2] Ensure all existing log statements include structured fields instead of only message strings

#### Integration & Testing

- [ ] T035 [US2] Initialize all observability components in main.rs (tracing, metrics, logging) before starting server
- [ ] T036 [US2] Implement graceful telemetry shutdown in main.rs to flush buffered data on application exit
- [ ] T037 [US2] Add integration test in tests/integration/observability_test.rs to verify telemetry export with mock OTLP collector
- [ ] T038 [US2] Create telemetry contract documentation in specs/003-production-observability/contracts/telemetry.md (spans, metrics, logs schemas)
- [ ] T039 [US2] Test telemetry with real OTLP collector following quickstart.md guide

**Checkpoint**: Full observability instrumentation complete - all requests generate traces, metrics, and structured logs

---

## Phase 5: User Story 3 - Observability Stack Deployment (Priority: P2)

**Goal**: Pre-configured observability stack (SigNoz or Grafana) deployable via Docker Compose with dashboards showing application metrics

**Independent Test**: Run `docker compose up` with observability stack, access UI, verify pre-configured dashboards show real application data without manual setup

### Implementation for User Story 3

- [ ] T040 [P] [US3] Create docker-compose.observability.yml with SigNoz stack (ClickHouse, OTEL Collector, Query Service, Frontend)
- [ ] T041 [P] [US3] Create docker-compose.prod.yml with complete production stack (app, PostgreSQL, SigNoz)
- [ ] T042 [US3] Configure OTEL Collector configuration in deployment/otel-collector-config.yaml with OTLP receivers and exporters
- [ ] T043 [US3] Add persistent volumes configuration in docker-compose files for data retention
- [ ] T044 [US3] Set resource limits in docker-compose files (CPU, memory) for observability services
- [ ] T045 [US3] Create SigNoz dashboard configuration in deployment/dashboards/quote-service.json with key metrics (request rate, error rate, latency, quotes generated)
- [ ] T046 [US3] Add health checks to all docker-compose services
- [ ] T047 [US3] Document observability stack access in docs/observability/dashboards.md (UI URLs, default credentials, navigation guide)
- [ ] T048 [US3] Create alternative Grafana stack docker-compose in deployment/docker-compose.grafana.yml (Tempo, Loki, Prometheus, Grafana)
- [ ] T049 [US3] Test observability stack deployment: start stack, verify all services healthy, access UI, confirm dashboards show data

**Checkpoint**: Observability stack can be deployed and shows real application telemetry data

---

## Phase 6: User Story 4 - Production Deployment Documentation (Priority: P2)

**Goal**: Comprehensive deployment guides and automation scripts for deploying to production on VPS, Docker, or managed platforms

**Independent Test**: New engineer follows documentation to deploy to test VPS, successfully deploys without prior codebase knowledge, all security checks pass

### Implementation for User Story 4

#### Deployment Scripts

- [ ] T050 [P] [US4] Create VPS deployment script in deployment/scripts/deploy-vps.sh (setup system dependencies, install Docker, configure firewall)
- [ ] T051 [P] [US4] Create database setup script in deployment/scripts/setup-db.sh (create database, run migrations, create db user)
- [ ] T052 [P] [US4] Create application deployment script in deployment/scripts/deploy-app.sh (build image, configure secrets, start services)
- [ ] T053 [P] [US4] Create nginx configuration template in deployment/nginx/site.conf with reverse proxy, SSL, and rate limiting

#### Documentation

- [ ] T054 [P] [US4] Write Docker Compose deployment guide in docs/deployment/docker-compose.md (prerequisites, configuration, deployment steps, verification)
- [ ] T055 [P] [US4] Write VPS deployment guide in docs/deployment/vps-ubuntu.md (Ubuntu 22.04 setup, nginx configuration, SSL certificates, systemd service)
- [ ] T056 [P] [US4] Write CleverCloud deployment guide in docs/deployment/clevercloud.md (platform-specific configuration, environment variables, addons)
- [ ] T057 [P] [US4] Create security checklist in docs/deployment/security-checklist.md (secrets management, HTTPS, database security, file permissions, rate limiting)
- [ ] T058 [P] [US4] Create troubleshooting guide in docs/deployment/troubleshooting.md (common issues: database connection, file permissions, port conflicts, SSL certificates)
- [ ] T059 [US4] Document minimum system requirements in docs/deployment/requirements.md (CPU, RAM, disk, network, software dependencies)

#### Testing & Validation

- [ ] T060 [US4] Enhance Dockerfile with healthcheck directive using /health endpoint
- [ ] T061 [US4] Test VPS deployment script on fresh Ubuntu 22.04 VM
- [ ] T062 [US4] Test Docker Compose production deployment locally
- [ ] T063 [US4] Verify security checklist items can be validated (secrets not in code/logs, HTTPS configured, etc.)

**Checkpoint**: Deployment documentation and automation complete - service can be deployed to any supported platform

---

## Phase 7: User Story 5 - Health and Readiness Endpoints (Priority: P3)

**Goal**: Standardized health check endpoints for load balancers and orchestration systems to detect service health

**Independent Test**: Call health endpoints, simulate failure scenarios (database down, filesystem full), verify appropriate status codes and orchestration system handling

### Implementation for User Story 5

- [ ] T064 [P] [US5] Create health check module in src/api/health.rs with health check logic
- [ ] T065 [P] [US5] Implement /health liveness endpoint in src/api/health.rs (returns 200 if service running)
- [ ] T066 [US5] Implement /ready readiness endpoint in src/api/health.rs (checks database connectivity and filesystem writability)
- [ ] T067 [US5] Create ReadinessResponse struct in src/api/health.rs with status and checks array
- [ ] T068 [US5] Add health check routes to router in src/api/routes.rs
- [ ] T069 [US5] Implement database connectivity check using sqlx pool.acquire()
- [ ] T070 [US5] Implement filesystem writability check by creating/deleting test file in upload directory
- [ ] T071 [US5] Add health check metrics to src/observability/metrics.rs (health_check_status, readiness_check_failures)
- [ ] T072 [US5] Record health check results as OpenTelemetry metrics in src/api/health.rs
- [ ] T073 [US5] Create integration test in tests/integration/health_checks_test.rs (test both endpoints, simulate failures)
- [ ] T074 [US5] Create OpenAPI specification in specs/003-production-observability/contracts/health.yaml for health endpoints
- [ ] T075 [US5] Update Docker healthcheck in Dockerfile to use /health endpoint
- [ ] T076 [US5] Test health checks with Docker: verify container reports healthy/unhealthy status correctly

**Checkpoint**: Health check endpoints complete and validated - orchestration systems can monitor service health

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

- [ ] T077 [P] Update README.md with observability quickstart section linking to docs
- [ ] T078 [P] Add observability architecture diagram to docs/observability/architecture.md
- [ ] T079 [P] Create runbook in docs/observability/runbook.md (common operational tasks: viewing traces, debugging issues, scaling stack)
- [ ] T080 Validate quickstart.md by following steps from scratch
- [ ] T081 Run full test suite (cargo test) and ensure all tests pass
- [ ] T082 Verify telemetry overhead < 5% p95 latency using load testing
- [ ] T083 [P] Add deployment CI/CD workflow in .github/workflows/deploy.yml
- [ ] T084 Security audit: verify no secrets in logs, traces, or version control

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3-7)**: All depend on Foundational phase completion
  - US1 (Configuration) and US2 (Observability) can proceed in parallel after Phase 2
  - US3 (Observability Stack) depends on US2 (instrumentation must exist to visualize)
  - US4 (Deployment Docs) can proceed in parallel with US2-US3
  - US5 (Health Checks) can proceed in parallel with US2-US4
- **Polish (Phase 8)**: Depends on completion of desired user stories

### User Story Dependencies

- **User Story 1 (P1 - Configuration)**: Can start after Foundational (Phase 2) - No dependencies on other stories
- **User Story 2 (P1 - Observability)**: Can start after Foundational (Phase 2) and US1 (needs config) - Independent otherwise
- **User Story 3 (P2 - Observability Stack)**: Requires US2 (instrumentation) complete to have data to visualize
- **User Story 4 (P2 - Deployment Docs)**: Can start after Foundational - No dependencies on other stories (documents all stories)
- **User Story 5 (P3 - Health Checks)**: Can start after Foundational - No dependencies on other stories

### Within Each User Story

**US1 (Configuration)**:
- Configuration structure (T009) before validation (T010)
- Core config before integration test (T014)

**US2 (Observability)**:
- Tracer initialization (T016-T017) before instrumentation (T018-T022)
- Metrics setup (T023) before metric recording (T024-T029)
- Logging config (T030-T031) before logging subscriber setup (T032)
- All instrumentation before integration tests (T037)

**US3 (Observability Stack)**:
- Docker Compose files (T040-T041) before collector config (T042)
- Stack configuration before dashboard setup (T045)
- All configuration before testing (T049)

**US4 (Deployment Docs)**:
- Scripts and docs can be written in parallel
- Testing (T060-T063) after respective artifacts created

**US5 (Health Checks)**:
- Health module (T064) before endpoint implementations (T065-T066)
- Check implementations (T069-T070) before readiness endpoint (T066)
- All endpoints before integration tests (T073)

### Parallel Opportunities

- **Phase 1**: T002, T003, T004 can run in parallel
- **Phase 2**: T006 can run in parallel after T005
- **US1**: T011, T012, T015 can run in parallel after T009-T010
- **US2 Tracing**: T016, T017 in parallel
- **US2 Metrics**: T023, T024, T025 in parallel
- **US2 Logging**: T030, T031 in parallel
- **US3**: T040, T041, T048 in parallel (different compose files)
- **US4**: All scripts (T050-T053) and docs (T054-T059) can be written in parallel
- **US5**: T064, T065 in parallel
- **Phase 8**: T077, T078, T079, T083 can run in parallel

---

## Parallel Example: User Story 2 (Observability Instrumentation)

```bash
# Launch tracing setup tasks together:
Task T016: "Implement OpenTelemetry tracer initialization in src/observability/tracing.rs"
Task T017: "Create tracing-opentelemetry bridge layer in src/observability/tracing.rs"

# Launch metrics setup tasks together:
Task T023: "Implement metrics setup in src/observability/metrics.rs"
Task T024: "Create business metrics in src/observability/metrics.rs"
Task T025: "Create technical metrics in src/observability/metrics.rs"

# Launch logging setup tasks together:
Task T030: "Implement structured JSON logging in src/observability/logging.rs (production)"
Task T031: "Implement human-readable logging in src/observability/logging.rs (development)"
```

---

## Implementation Strategy

### MVP First (User Stories 1 + 2 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational
3. Complete Phase 3: User Story 1 (Configuration)
4. Complete Phase 4: User Story 2 (Observability)
5. **STOP and VALIDATE**: Deploy to test environment, generate traffic, verify traces/metrics/logs in OTLP collector
6. MVP Complete: Service has production-ready configuration and full observability

### Incremental Delivery

1. Complete Setup + Foundational → Foundation ready
2. Add US1 (Configuration) → Test independently → Service can be deployed with env vars ✅
3. Add US2 (Observability) → Test independently → Full telemetry working ✅ (MVP!)
4. Add US3 (Observability Stack) → Test independently → Self-hosted visualization ✅
5. Add US4 (Deployment Docs) → Test independently → Repeatable deployments ✅
6. Add US5 (Health Checks) → Test independently → Orchestration-ready ✅
7. Each story adds value without breaking previous stories

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together
2. Once Foundational is done:
   - Developer A: US1 (Configuration) then US5 (Health Checks)
   - Developer B: US2 (Observability Instrumentation)
   - Developer C: US4 (Deployment Docs)
3. After US2 complete:
   - Developer B or C: US3 (Observability Stack)
4. Stories integrate independently with minimal conflicts

---

## Notes

- [P] tasks = different files, no dependencies - can run in parallel
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- Tests are not required for this feature (infrastructure code) but integration tests recommended for critical paths
- Avoid: vague tasks, same file conflicts, cross-story dependencies that break independence

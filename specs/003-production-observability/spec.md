# Feature Specification: Production Deployment and Observability

**Feature Branch**: `003-production-observability`
**Created**: 2025-12-24
**Status**: Draft
**Input**: User description: "Production Deployment and Observability with OpenTelemetry - Prepare 3D print quote service for production deployment with modern monitoring using OpenTelemetry"

## User Scenarios & Testing

### User Story 1 - Production Configuration Management (Priority: P1)

As an operations engineer, I want to deploy the service with secure, environment-specific configuration so that I can safely run the service in production across different platforms (VPS, VM, or managed services like CleverCloud) without hardcoded secrets.

**Why this priority**: This is foundational for any production deployment. Without proper configuration management, the service cannot be deployed securely or flexibly to different environments.

**Independent Test**: Can be fully tested by deploying to a test environment with environment variables configured, verifying that all secrets are loaded correctly, and confirming that different configurations work for dev vs prod modes without code changes.

**Acceptance Scenarios**:

1. **Given** a fresh production server, **When** I set required environment variables (DATABASE_URL, ADMIN_TOKEN, MCP_TOKEN), **Then** the service starts successfully and connects to the database
2. **Given** environment is set to "production", **When** the service runs, **Then** logs are in JSON format and verbose debug logging is disabled
3. **Given** different deployment platforms (Docker, bare metal, CleverCloud), **When** I configure via environment variables, **Then** the service works identically across all platforms
4. **Given** missing required secrets, **When** the service starts, **Then** it fails fast with clear error messages indicating which configuration is missing

---

### User Story 2 - Application Observability with OpenTelemetry (Priority: P1)

As a developer or operations engineer, I want comprehensive observability instrumentation using OpenTelemetry so that I can understand system behavior, diagnose issues, and monitor performance in production without vendor lock-in.

**Why this priority**: Observability is critical for running services in production. Without it, debugging issues and understanding system health is impossible. This must be in place before any production deployment.

**Independent Test**: Can be tested by running the instrumented application, generating different types of requests (upload files, generate quotes, admin operations), and verifying that traces, metrics, and structured logs appear in the OpenTelemetry collector with proper context propagation.

**Acceptance Scenarios**:

1. **Given** a user uploads a 3D file, **When** the request is processed, **Then** a complete trace is generated showing file upload → validation → volume calculation → price computation with timing for each step
2. **Given** multiple concurrent quote requests, **When** viewing metrics, **Then** I can see business metrics (quotes generated, average file size, calculation time) and technical metrics (endpoint latency, error rates, database connection pool usage)
3. **Given** an error occurs during file processing, **When** reviewing logs, **Then** I see structured JSON logs with trace_id linking to the trace, severity level, error details, and request context
4. **Given** the session cleanup job runs, **When** viewing traces, **Then** I can see how many sessions were cleaned, files deleted, and time taken
5. **Given** OpenTelemetry is configured with OTLP endpoint, **When** the service runs, **Then** all telemetry data is exported to the configured collector without data loss

---

### User Story 3 - Observability Stack Deployment (Priority: P2)

As an operations engineer, I want a pre-configured observability stack (using SigNoz or Grafana ecosystem) that I can deploy alongside the application so that I have immediate visibility into system health without manual dashboard configuration.

**Why this priority**: While instrumentation (P1) is critical, having a visualization layer makes the data actionable. This is P2 because you could export to external services initially, but having a self-hosted stack improves the deployment experience.

**Independent Test**: Can be tested by running `docker compose up` with the observability stack, accessing the UI, and verifying that pre-configured dashboards show real data from the running application without manual setup.

**Acceptance Scenarios**:

1. **Given** Docker and Docker Compose installed, **When** I run the observability stack docker-compose file, **Then** all services (app, database, observability backend) start successfully and are accessible
2. **Given** the observability stack is running, **When** I access the web UI, **Then** I see pre-configured dashboards showing key metrics: request rate, error rate, latency percentiles, quote generation metrics, database performance
3. **Given** the application processes requests, **When** I search for traces in the UI, **Then** I can filter by service, endpoint, duration, and see complete traces with spans from all components
4. **Given** production deployment on a VPS, **When** using the docker-compose stack, **Then** it uses reasonable resource limits and doesn't overwhelm the server
5. **Given** the observability stack configuration, **When** I review it, **Then** it uses persistent volumes for data retention and includes backup/restore instructions

---

### User Story 4 - Production Deployment Documentation (Priority: P2)

As an operations engineer, I want comprehensive deployment guides and automation scripts so that I can confidently deploy the service to production on my chosen platform with security best practices.

**Why this priority**: P2 because you need the instrumentation (P1) and configuration (P1) first. This makes deployment repeatable and reduces errors, but the service can technically be deployed manually before this exists.

**Independent Test**: Can be tested by a new engineer following the documentation to deploy to a test VPS/VM, verifying that they can successfully deploy without prior knowledge of the codebase, and that all security checks pass.

**Acceptance Scenarios**:

1. **Given** a fresh Ubuntu 22.04 VPS, **When** I follow the deployment guide, **Then** I can deploy the service with nginx reverse proxy, SSL certificates, and all components running within 30 minutes
2. **Given** deployment scripts provided, **When** I run the automated deployment, **Then** it sets up the database, configures secrets securely, builds/deploys the application, and runs health checks
3. **Given** the security checklist, **When** I review my production deployment, **Then** I can verify: secrets not in code/logs, HTTPS enabled, database access restricted, file upload limits enforced, rate limiting active
4. **Given** a deployment issue, **When** I consult the troubleshooting guide, **Then** I find solutions for common problems: database connection failures, file permission issues, port conflicts, SSL certificate problems
5. **Given** multiple deployment platforms documented (Docker Compose, VPS, CleverCloud), **When** I choose one, **Then** I have platform-specific instructions and configuration examples

---

### User Story 5 - Health and Readiness Endpoints (Priority: P3)

As a load balancer or orchestration system, I want standardized health check endpoints so that I can automatically detect service health, route traffic appropriately, and restart unhealthy instances.

**Why this priority**: P3 because this enhances operational maturity but isn't required for initial production deployment. You can run without automated health checks initially, relying on observability (P1) to detect issues.

**Independent Test**: Can be tested by calling the health endpoints, simulating various failure scenarios (database down, filesystem full), and verifying that the endpoints return appropriate status codes and that container orchestration systems correctly handle the responses.

**Acceptance Scenarios**:

1. **Given** the service is running normally, **When** /health is called, **Then** it returns HTTP 200 with basic service status
2. **Given** the service is running but database is unreachable, **When** /ready is called, **Then** it returns HTTP 503 and includes details about which check failed (database connection)
3. **Given** the upload directory is not writable, **When** /ready is called, **Then** it returns HTTP 503 indicating filesystem check failure
4. **Given** a Docker container with health check configured, **When** the service becomes unhealthy, **Then** the container reports unhealthy status and orchestrator can restart it
5. **Given** health check metrics enabled, **When** viewing OpenTelemetry metrics, **Then** I see health status history and can track availability over time

---

### Edge Cases

- What happens when the OpenTelemetry collector is unavailable? (Service should continue operating, buffering telemetry data temporarily or dropping it gracefully without affecting user requests)
- How does the system handle very high cardinality in traces (e.g., unique session IDs in every span)? (Use sampling strategies to avoid overwhelming the observability backend)
- What if environment variables are partially configured (some secrets present, others missing)? (Service fails fast at startup with clear error messages listing missing configuration)
- How does log volume scale with high traffic? (Implement log sampling or rate limiting for high-frequency logs; structure logs for efficient querying)
- What if the database connection pool is exhausted? (OpenTelemetry metrics show pool saturation; health checks may fail; service returns 503 with appropriate error)
- How do we handle timezone differences in logs and traces? (All timestamps use UTC; documentation explains timezone handling)
- What if the observability stack runs out of disk space? (Configure retention policies and data rotation; monitor disk usage via metrics; implement alerts)

## Requirements

### Functional Requirements

#### Configuration (US1)

- **FR-001**: System MUST load all configuration from environment variables (DATABASE_URL, ADMIN_TOKEN, MCP_TOKEN, OTEL_EXPORTER_OTLP_ENDPOINT, ENVIRONMENT)
- **FR-002**: System MUST support both development and production configuration modes controlled by ENVIRONMENT variable (development, production)
- **FR-003**: System MUST fail fast at startup if required environment variables are missing, with clear error messages
- **FR-004**: System MUST use different log formats based on environment (human-readable for dev, JSON for production)
- **FR-005**: System MUST document all environment variables with required/optional status, defaults, and examples
- **FR-006**: System MUST NOT log sensitive data (passwords, tokens, API keys) in any environment

#### OpenTelemetry Instrumentation (US2)

- **FR-007**: System MUST automatically instrument all HTTP requests with OpenTelemetry traces including method, path, status code, and duration
- **FR-008**: System MUST create custom traces for key operations: file upload, 3D file processing, quote calculation, session cleanup
- **FR-009**: System MUST record business metrics: total quotes generated, average file size, calculation time distribution, material usage
- **FR-010**: System MUST record technical metrics: HTTP request rate, error rate by endpoint, response time percentiles (p50, p95, p99), database connection pool stats
- **FR-011**: System MUST output structured logs in JSON format with fields: timestamp, severity, message, trace_id, span_id, service.name
- **FR-012**: System MUST propagate trace context across all operations within a request
- **FR-013**: System MUST export telemetry via OTLP protocol to configurable endpoint
- **FR-014**: System MUST gracefully handle telemetry export failures without impacting application availability

#### Observability Stack (US3)

- **FR-015**: System MUST provide Docker Compose configuration for running application with observability backend
- **FR-016**: Observability stack MUST include one of: SigNoz (all-in-one) OR Grafana Tempo + Loki + Prometheus + Grafana
- **FR-017**: Observability stack MUST include pre-configured dashboards showing: request rates, error rates, latency distributions, business metrics
- **FR-018**: Observability stack MUST persist data across container restarts
- **FR-019**: Observability stack MUST include documentation for accessing UI, querying data, and basic troubleshooting

#### Deployment (US4)

- **FR-020**: System MUST provide deployment guide for Ubuntu VPS/VM including nginx reverse proxy setup
- **FR-021**: System MUST provide deployment guide for Docker Compose production deployment
- **FR-022**: System MUST provide automated deployment scripts that handle database setup, secret configuration, and application deployment
- **FR-023**: System MUST document security checklist covering: secret management, HTTPS, database security, file permissions, rate limiting
- **FR-024**: System MUST provide troubleshooting guide for common deployment issues
- **FR-025**: Deployment documentation MUST specify minimum system requirements (CPU, RAM, disk, network)

#### Health Checks (US5)

- **FR-026**: System MUST expose /health endpoint returning HTTP 200 if service is running
- **FR-027**: System MUST expose /ready endpoint that checks database connectivity and filesystem accessibility
- **FR-028**: /ready endpoint MUST return HTTP 200 if all checks pass, HTTP 503 if any check fails, with details about failed checks
- **FR-029**: System MUST support Docker healthcheck configuration using /health endpoint
- **FR-030**: System MUST record health check results as OpenTelemetry metrics (availability percentage over time)

### Key Entities

*This feature primarily enhances the existing system with observability and deployment capabilities. No new data entities are introduced, but telemetry data is generated:*

- **Trace**: Represents a complete request flow through the system, containing multiple spans (e.g., upload trace containing spans for validation, file parsing, volume calculation)
- **Span**: Represents a single operation within a trace, including start time, duration, attributes, and status
- **Metric**: Represents a measurement over time (counters, gauges, histograms) for both business and technical indicators
- **Log Entry**: Structured record of an event with timestamp, severity, message, and trace context

## Success Criteria

### Measurable Outcomes

- **SC-001**: Operations engineer can deploy the service to a production environment in under 1 hour using provided documentation and scripts
- **SC-002**: 100% of application requests generate traces that can be viewed in the observability UI within 5 seconds
- **SC-003**: All critical operations (file upload, quote generation, cleanup) have custom traces showing detailed timing breakdowns
- **SC-004**: Pre-configured dashboards show real-time data for: request rate, error rate, p95 latency, active quotes, database health
- **SC-005**: Service continues operating normally even when observability backend is unavailable (telemetry failures don't impact users)
- **SC-006**: Health check endpoints respond within 100ms under normal conditions
- **SC-007**: Operations engineer can identify the root cause of a failed request within 5 minutes using traces and logs
- **SC-008**: Business stakeholders can view quote generation metrics (count, average size, success rate) without technical knowledge
- **SC-009**: Zero secrets or sensitive data appear in logs or traces
- **SC-010**: Service can be deployed to different platforms (Docker, VPS, managed services) using platform-specific guides without code changes

## Assumptions

- The current service already has basic tracing via Rust's `tracing` crate, which can be integrated with OpenTelemetry
- The target deployment platforms support Docker and standard Linux environments (Ubuntu 22.04+)
- Operations engineers have basic Docker and Linux administration skills
- The service will be deployed behind a reverse proxy (nginx) for HTTPS termination
- OpenTelemetry collector or compatible backend will be available to receive OTLP exports
- Initial observability stack will be self-hosted (not using commercial SaaS initially)
- The service currently uses PostgreSQL for data persistence
- Current traffic is expected to be moderate (< 1000 requests/minute initially), allowing for detailed tracing without sampling

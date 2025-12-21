# Feature Specification: 3D Printing Quote Service

**Feature Branch**: `001-3d-print-quoting`
**Created**: 2025-11-15
**Status**: Draft
**Input**: User description: "Web application for 3D printing quote service with file upload, visualization, material selection, and pricing"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Upload and Visualize 3D Files (Priority: P1)

As a customer, I want to upload one or more 3D model files (STL or 3MF format) and see them visualized in my browser so that I can verify I'm submitting the correct models for printing.

**Why this priority**: This is the core entry point to the service. Without file upload and visualization, users cannot begin the quoting process. This delivers immediate value by allowing users to confirm their 3D models visually.

**Independent Test**: Can be fully tested by uploading various STL/3MF files and verifying they render correctly in the browser, delivering immediate visual confirmation value.

**Acceptance Scenarios**:

1. **Given** I am on the upload page, **When** I drag and drop a single STL file, **Then** the file uploads and displays a 3D preview within 5 seconds
2. **Given** I am on the upload page, **When** I select multiple 3MF files via file browser, **Then** all files upload and each displays its own 3D preview
3. **Given** I have uploaded files, **When** I interact with a preview, **Then** I can rotate, zoom, and pan the 3D model
4. **Given** I upload an invalid file type, **When** the upload completes, **Then** I see a clear error message explaining accepted formats

---

### User Story 2 - Configure Print Options per Model (Priority: P2)

As a customer, I want to select the material and other printing parameters for each uploaded model so that I can customize the print job to my specific needs.

**Why this priority**: Configuration options are essential for accurate pricing and fulfilling customer requirements. This builds on P1 by adding customization capabilities.

**Independent Test**: Can be fully tested by uploading a file and selecting different materials/options, verifying the selections are saved and displayed correctly.

**Acceptance Scenarios**:

1. **Given** I have uploaded a 3D model, **When** I view the configuration panel, **Then** I see available material options with descriptions
2. **Given** I am configuring a model, **When** I select a material, **Then** the selection is saved and visually confirmed
3. **Given** I have multiple uploaded models, **When** I configure each one, **Then** I can set different materials for each model independently
4. **Given** I select a material, **When** the system calculates, **Then** I see an immediate estimate update reflecting the material choice

---

### User Story 3 - Receive Instant Price Quote (Priority: P3)

As a customer, I want to see an instant price quote for my configured print job so that I can decide whether to proceed with the order.

**Why this priority**: Pricing is the ultimate goal of the application. This completes the core user journey by providing the business value of automated quoting.

**Independent Test**: Can be fully tested by uploading files, configuring options, and verifying the calculated price matches expected pricing rules.

**Acceptance Scenarios**:

1. **Given** I have configured at least one model, **When** I request a quote, **Then** I see a detailed price breakdown within 3 seconds
2. **Given** I have multiple configured models, **When** I view the quote, **Then** I see individual prices per model and a total sum
3. **Given** I change a material selection, **When** the system recalculates, **Then** the price updates automatically
4. **Given** the quote is displayed, **When** I review it, **Then** I see clear itemization of costs (material, volume, processing fees)

---

### User Story 4 - Administrator Manages Pricing (Priority: P4)

As an administrator, I want to update material prices and pricing formulas easily so that I can adjust business rates without technical intervention.

**Why this priority**: Enables business agility by allowing non-technical price adjustments. Essential for long-term viability but not required for initial customer-facing MVP.

**Independent Test**: Can be fully tested by logging in as admin, modifying prices, and verifying customer quotes reflect new pricing.

**Acceptance Scenarios**:

1. **Given** I am logged in as administrator, **When** I access the pricing management interface, **Then** I see all configurable materials and their current prices
2. **Given** I am in pricing management, **When** I update a material price, **Then** the change takes effect immediately for new quotes
3. **Given** I modify pricing parameters, **When** I save changes, **Then** I receive confirmation and can see the change history
4. **Given** I need to add a new material option, **When** I create it in the admin interface, **Then** it becomes available to customers immediately

---

### User Story 5 - MCP Integration for AI Models (Priority: P5)

As an AI model or automation system, I want to generate quotes programmatically via MCP (Model Context Protocol) so that I can integrate quote generation into automated workflows without using the web interface.

**Why this priority**: Enables AI models and automation tools to leverage the quote service programmatically. This extends the platform's reach beyond human users to AI assistants and automated systems.

**Independent Test**: Can be fully tested by using an MCP client to upload a file, configure materials, and receive a quote entirely through protocol commands.

**Acceptance Scenarios**:

1. **Given** an MCP client is connected to the server, **When** it calls the upload_model tool with base64-encoded file data, **Then** it receives a model_id and model metadata
2. **Given** an MCP client has uploaded a model, **When** it calls the list_materials tool, **Then** it receives a list of available materials with prices
3. **Given** an MCP client has a model_id, **When** it calls the configure_model tool with material selection, **Then** the configuration is saved and confirmed
4. **Given** an MCP client has configured models, **When** it calls the generate_quote tool, **Then** it receives a complete quote with itemized breakdown

---

### User Story 6 - Webhook System for External Integrations (Priority: P6)

As a system administrator, I want to configure webhooks that export quotes to external platforms (Notion, Obsidian, Odoo, etc.) so that I can integrate the quote service into my existing workflow tools without manual copy-paste.

**Why this priority**: Enables seamless integration with external business tools and CRM systems. This makes the quote service part of a larger ecosystem rather than a standalone tool.

**Independent Test**: Can be fully tested by configuring a webhook URL, generating a quote, and verifying the quote data is sent to the external endpoint with correct format and retry logic.

**Acceptance Scenarios**:

1. **Given** I am an administrator, **When** I configure a webhook URL with authentication headers, **Then** the configuration is saved and can be tested
2. **Given** a webhook is configured, **When** a quote is generated, **Then** the quote data is sent to the webhook URL with proper formatting
3. **Given** a webhook call fails, **When** the system retries, **Then** it follows exponential backoff and logs the failure
4. **Given** I want to customize exports, **When** I configure webhook payload templates, **Then** the quote data is transformed according to the template before sending
5. **Given** I have multiple webhooks configured, **When** a quote is generated, **Then** all active webhooks receive the quote data in parallel

---

### Edge Cases

- What happens when a user uploads a corrupted or malformed STL/3MF file?
- How does the system handle extremely large files (>100MB)?
- What happens when a user uploads a model with zero volume (flat surface)?
- How does pricing handle models with very complex geometries requiring extended print times?
- What happens when a user's session expires mid-configuration?
- How does the system behave when pricing data is unavailable or misconfigured?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST accept file uploads in STL and 3MF formats only
- **FR-002**: System MUST validate uploaded files for format integrity before processing
- **FR-003**: System MUST render interactive 3D previews of uploaded models in the browser
- **FR-004**: System MUST support uploading multiple files in a single session (minimum 10 files)
- **FR-005**: System MUST calculate and display model dimensions (length, width, height) automatically
- **FR-006**: System MUST calculate model volume for pricing purposes
- **FR-007**: System MUST provide a selectable list of available printing materials
- **FR-008**: System MUST allow users to assign different materials to different models
- **FR-009**: System MUST calculate pricing based on model volume and selected material
- **FR-010**: System MUST display itemized price breakdown (material cost, volume cost, any additional fees)
- **FR-011**: System MUST update price calculations in real-time as users modify selections
- **FR-012**: System MUST provide an administrative interface for managing material options
- **FR-013**: System MUST allow administrators to update material prices without code changes
- **FR-014**: System MUST persist pricing configuration changes immediately
- **FR-015**: System MUST support extensible service types to accommodate future services (laser cutting, engraving)
- **FR-016**: System MUST maintain separation between different service type configurations
- **FR-017**: System MUST handle file upload failures gracefully with user-friendly error messages
- **FR-018**: System MUST limit individual file size to 50MB maximum
- **FR-019**: System MUST provide visual feedback during file upload and processing
- **FR-020**: System MUST allow users to remove uploaded files from their quote session
- **FR-021**: System MUST provide an MCP (Model Context Protocol) server for programmatic access
- **FR-022**: System MUST expose MCP tools for file upload, material listing, model configuration, and quote generation
- **FR-023**: System MUST validate and sanitize MCP requests with same security as web API
- **FR-024**: System MUST support base64-encoded file uploads via MCP protocol
- **FR-025**: System MUST return structured JSON responses conforming to MCP specification
- **FR-026**: System MUST provide webhook configuration interface for administrators
- **FR-027**: System MUST trigger webhooks when quotes are generated or updated
- **FR-028**: System MUST support customizable webhook payload templates
- **FR-029**: System MUST implement retry logic with exponential backoff for failed webhook calls
- **FR-030**: System MUST log all webhook attempts with success/failure status
- **FR-031**: System MUST support authentication headers (API keys, Bearer tokens) for webhooks
- **FR-032**: System MUST allow webhook testing before activation

### Key Entities

- **3D Model**: Represents an uploaded file with calculated properties (volume, dimensions, file format) and user-selected configuration (material choice)
- **Material**: A printing material option with associated properties (name, description, price per unit volume, availability status)
- **Quote**: A collection of configured 3D models with calculated total price, itemized breakdown, and timestamp
- **Service Type**: Category of manufacturing service (3D printing, laser cutting, engraving) that groups related materials and pricing rules
- **Pricing Configuration**: Administrative settings including material prices, volume multipliers, and service fees that can be modified without code deployment
- **Quote Session**: Temporary user workspace containing uploaded models and configurations, persisted until quote completion or session expiration

## Assumptions

- Users have modern web browsers with WebGL support for 3D rendering
- File uploads are temporary and not stored long-term (session-based)
- No user authentication required for generating quotes (anonymous quoting)
- Pricing formula is primarily volume-based with material multipliers
- System operates in single currency (EUR assumed for French market)
- Maximum session duration of 24 hours before automatic cleanup
- Models are solid objects (not hollow) for volume calculation purposes

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can upload and visualize a 3D model in under 30 seconds for files up to 10MB
- **SC-002**: 95% of valid STL/3MF files render correctly in the 3D preview
- **SC-003**: Users can complete the full quote process (upload → configure → receive price) in under 3 minutes
- **SC-004**: Price calculations complete within 2 seconds of configuration changes
- **SC-005**: Administrators can update material prices in under 1 minute without technical assistance
- **SC-006**: System maintains pricing accuracy to 2 decimal places with no rounding errors
- **SC-007**: Application supports 100 concurrent users generating quotes without performance degradation
- **SC-008**: 90% of users successfully obtain a quote on their first attempt
- **SC-009**: System properly rejects 100% of invalid file formats with clear error messages
- **SC-010**: New service types (laser cutting, engraving) can be added to the platform without restructuring existing functionality

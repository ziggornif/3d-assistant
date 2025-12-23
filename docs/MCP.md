# MCP (Model Context Protocol) Integration

This document describes the MCP server integration in the 3D Print Quote Service, which allows AI models and automation tools to generate quotes programmatically without using the web interface.

## Overview

The MCP server provides a set of tools that enable:
- Uploading 3D model files (STL/3MF) via base64-encoded data
- Listing available materials and their prices
- Configuring models with materials and quantities
- Generating complete quotes for sessions

## Architecture

The MCP implementation is built using the [`rmcp`](https://crates.io/crates/rmcp) library and consists of:

- **`src/mcp/quote_tools.rs`**: Tool definitions and business logic
- **`src/mcp/server.rs`**: MCP server setup and configuration
- **`src/mcp/mod.rs`**: Module exports

The MCP server is integrated into the main application and exposed at:
```
POST /mcp
```

## Available Tools

### 1. `list_materials`

Lists all available printing materials with their prices.

**Parameters:** None

**Returns:**
```json
[
  {
    "id": "pla_standard",
    "name": "PLA Standard",
    "base_price_per_cm3": 0.15,
    "active": true
  },
  {
    "id": "abs_standard",
    "name": "ABS Standard",
    "base_price_per_cm3": 0.18,
    "active": true
  }
]
```

### 2. `upload_model`

Uploads a 3D model file from base64-encoded data.

**Parameters:**
```json
{
  "session_id": "01JGXXX...",
  "filename": "cube.stl",
  "file_data": "base64-encoded-file-content"
}
```

**Returns:**
```json
{
  "model_id": "01JGYYY...",
  "filename": "cube.stl",
  "file_format": "stl",
  "volume_cm3": 8.0,
  "dimensions_mm": {
    "x": 20.0,
    "y": 20.0,
    "z": 20.0
  },
  "triangle_count": 12
}
```

**Validation:**
- File size must not exceed configured maximum (default 50MB)
- File must be valid STL or 3MF format
- Filename is automatically sanitized (dangerous characters removed)

**Processing:**
- Volume is calculated from triangle mesh
- Dimensions (bounding box) are extracted
- Triangle count is computed
- File is saved to disk in session-specific directory

### 3. `configure_model`

Configures a model with material selection and quantity.

**Parameters:**
```json
{
  "session_id": "01JGXXX...",
  "model_id": "01JGYYY...",
  "material_id": "pla_standard",
  "quantity": 5
}
```

**Returns:**
```json
{
  "model_id": "01JGYYY...",
  "material_id": "pla_standard",
  "quantity": 5,
  "estimated_price": 12.50
}
```

**Validation:**
- Model must exist in the specified session
- Material must exist and be active
- Quantity must be positive

### 4. `generate_quote`

Generates a complete quote for all configured models in a session.

**Parameters:**
```json
{
  "session_id": "01JGXXX..."
}
```

**Returns:**
```json
{
  "quote_id": "01JGZZZ...",
  "session_id": "01JGXXX...",
  "items": [
    {
      "model_id": "01JGYYY...",
      "filename": "cube.stl",
      "material_name": "PLA Standard",
      "quantity": 1,
      "unit_price": 2.50,
      "line_total": 2.50
    }
  ],
  "subtotal": 2.50,
  "total": 2.50
}
```

**Validation:**
- Session must have at least one model
- All models must be configured with a material
- Quote is persisted in the database

## Usage Example

Here's a typical workflow using the MCP tools:

### Step 1: Create a session
First, create a session via the REST API:
```bash
curl -X POST http://localhost:3000/api/sessions
```

Response:
```json
{
  "session_id": "01JGXXX123",
  "expires_at": "2025-12-24T15:30:00Z"
}
```

### Step 2: List available materials (MCP)
```json
{
  "method": "tools/call",
  "params": {
    "name": "list_materials"
  }
}
```

### Step 3: Upload a model (MCP)
```bash
# First, encode your STL file to base64
base64 cube.stl > cube_base64.txt
```

```json
{
  "method": "tools/call",
  "params": {
    "name": "upload_model",
    "arguments": {
      "session_id": "01JGXXX123",
      "filename": "cube.stl",
      "file_data": "<base64-content>"
    }
  }
}
```

### Step 4: Configure the model (MCP)
```json
{
  "method": "tools/call",
  "params": {
    "name": "configure_model",
    "arguments": {
      "session_id": "01JGXXX123",
      "model_id": "01JGYYY456",
      "material_id": "pla_standard",
      "quantity": 5
    }
  }
}
```

### Step 5: Generate the quote (MCP)
```json
{
  "method": "tools/call",
  "params": {
    "name": "generate_quote",
    "arguments": {
      "session_id": "01JGXXX123"
    }
  }
}
```

## Testing

Integration tests are available in `tests/mcp_integration_test.rs`:

```bash
# Run MCP integration tests
cargo test --test mcp_integration_test

# Run all tests
cargo test
```

The tests cover:
- Material listing
- STL file upload with volume calculation
- Model configuration with materials
- Quote generation
- Error handling (invalid session, unconfigured models, etc.)

## Configuration

The MCP server uses the same configuration as the main application:

```env
# .env
DATABASE_URL=postgres://user:password@localhost:5432/quotes
UPLOAD_DIR=./uploads
MAX_FILE_SIZE_MB=50
```

## Security Considerations

1. **File Size Limits**: Upload size is enforced (default 50MB) to prevent DoS attacks
2. **File Validation**: All uploaded files are validated for correct format (STL/3MF)
3. **Filename Sanitization**: Dangerous characters and path traversal attempts are removed
4. **Session Isolation**: Models are isolated per session in separate directories
5. **Material Validation**: Only active materials can be used for configuration

## Implementation Details

### Technology Stack
- **rmcp**: Rust MCP implementation with declarative macros
- **sqlx**: Database operations with compile-time query verification
- **base64**: File encoding/decoding
- **serde**: JSON serialization/deserialization
- **schemars**: JSON Schema generation for tool parameters

### Macros Used
- `#[tool_router]`: Defines the tool router for the QuoteTools struct
- `#[tool]`: Marks methods as MCP tools with descriptions
- `#[tool_handler]`: Implements ServerHandler trait for the tools

### Data Flow
```
Client (AI Model/Tool)
  ↓ MCP Request
POST /mcp
  ↓
rmcp StreamableHttpService
  ↓
QuoteTools (tool execution)
  ↓ Database Operations
PostgreSQL
  ↓ File System
./uploads/{session_id}/
```

## Error Handling

All tools return `Result<String, String>` where:
- **Ok(String)**: JSON-serialized success result
- **Err(String)**: Human-readable error message

Common errors:
- `"Session not found"`: Invalid or expired session ID
- `"Invalid base64 encoding"`: Malformed file data
- `"File size exceeds maximum"`: File too large
- `"File validation failed"`: Invalid STL/3MF format
- `"Model not found in session"`: Invalid model ID
- `"Material not found"`: Invalid material ID
- `"Material is not active"`: Inactive material selected
- `"No models found in session"`: Empty session for quote generation
- `"X model(s) missing material configuration"`: Unconfigured models

## Limitations

1. **No Authentication**: Currently no authentication on MCP endpoint (add in production)
2. **Single Quantity**: Quantity parameter is accepted but not fully implemented in quote calculation
3. **No Preview URLs**: MCP uploads don't generate 3D preview images
4. **Synchronous Processing**: File processing is synchronous (may timeout on very large files)

## Future Enhancements

- [ ] Add authentication/authorization for MCP endpoint
- [ ] Support streaming for large file uploads
- [ ] Generate 3D preview images for MCP uploads
- [ ] Add tool for listing session models
- [ ] Add tool for deleting models
- [ ] Support batch operations (upload multiple models)
- [ ] Add support for model metadata (notes, customer info)
- [ ] Webhook notifications on quote generation

## References

- [Model Context Protocol Specification](https://modelcontextprotocol.io/)
- [rmcp Rust Implementation](https://github.com/EmilLindfors/rmcp)
- [MCP Integration Tests](../tests/mcp_integration_test.rs)
- [Quote Tools Source](../src/mcp/quote_tools.rs)

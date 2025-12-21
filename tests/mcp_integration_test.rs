//! Integration tests for MCP (Model Context Protocol) implementation
//!
//! These tests verify the complete MCP workflow:
//! 1. JSON-RPC protocol compliance
//! 2. Tool schema validation
//! 3. End-to-end quote generation workflow
//! 4. Error handling and edge cases

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use quote_service::{
    api::routes::create_router,
    config::Config,
    db::create_pool,
    mcp::server::{JsonRpcRequest, JsonRpcResponse},
};
use serde_json::{json, Value};
use sqlx::PgPool;
use tempfile::TempDir;
use tower::ServiceExt;

/// Test helper to create test database pool
async fn setup_test_db() -> PgPool {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost/quote_service_test".to_string());

    create_pool(&database_url, 5)
        .await
        .expect("Failed to create test database pool")
}

/// Test helper to create test config
fn setup_test_config(temp_dir: &TempDir) -> Config {
    Config {
        database_url: std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost/quote_service_test".to_string()),
        host: "127.0.0.1".to_string(),
        port: 8080,
        upload_dir: temp_dir.path().join("uploads").to_string_lossy().to_string(),
        static_dir: temp_dir.path().join("static").to_string_lossy().to_string(),
        template_dir: temp_dir.path().join("templates").to_string_lossy().to_string(),
        max_file_size_bytes: 100 * 1024 * 1024,
        admin_token: "test-admin-token".to_string(),
        session_expiry_hours: 24,
        environment: "test".to_string(),
    }
}

/// Test helper to send MCP JSON-RPC request
async fn send_mcp_request(
    app: &mut axum::Router,
    request: JsonRpcRequest,
) -> (StatusCode, JsonRpcResponse) {
    let body = serde_json::to_string(&request).unwrap();

    let response = app
        .oneshot(
            Request::builder()
                .uri("/mcp")
                .method("POST")
                .header("content-type", "application/json")
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json_response: JsonRpcResponse = serde_json::from_slice(&body).unwrap();

    (status, json_response)
}

#[tokio::test]
async fn test_mcp_tools_list() {
    let pool = setup_test_db().await;
    let temp_dir = TempDir::new().unwrap();
    let config = setup_test_config(&temp_dir);

    let mut app = create_router(pool, config);

    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        method: "tools/list".to_string(),
        params: None,
    };

    let (status, response) = send_mcp_request(&mut app, request).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(response.jsonrpc, "2.0");
    assert!(response.error.is_none());

    let result = response.result.unwrap();
    let tools = result["tools"].as_array().unwrap();

    // Verify all 4 tools are present
    assert_eq!(tools.len(), 4);

    let tool_names: Vec<&str> = tools
        .iter()
        .map(|t| t["name"].as_str().unwrap())
        .collect();

    assert!(tool_names.contains(&"upload_model"));
    assert!(tool_names.contains(&"list_materials"));
    assert!(tool_names.contains(&"configure_model"));
    assert!(tool_names.contains(&"generate_quote"));

    // Verify each tool has proper schema
    for tool in tools {
        assert!(tool["name"].is_string());
        assert!(tool["description"].is_string());
        assert!(tool["input_schema"].is_object());
        assert!(tool["input_schema"]["type"].is_string());
    }
}

#[tokio::test]
async fn test_mcp_invalid_method() {
    let pool = setup_test_db().await;
    let temp_dir = TempDir::new().unwrap();
    let config = setup_test_config(&temp_dir);

    let mut app = create_router(pool, config);

    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        method: "invalid/method".to_string(),
        params: None,
    };

    let (status, response) = send_mcp_request(&mut app, request).await;

    assert_eq!(status, StatusCode::OK);
    assert!(response.result.is_none());

    let error = response.error.unwrap();
    assert_eq!(error.code, -32601); // Method not found
    assert!(error.message.contains("Method not found"));
}

#[tokio::test]
async fn test_mcp_list_materials() {
    let pool = setup_test_db().await;
    let temp_dir = TempDir::new().unwrap();
    let config = setup_test_config(&temp_dir);

    let mut app = create_router(pool, config);

    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(2)),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "list_materials",
            "arguments": {}
        })),
    };

    let (status, response) = send_mcp_request(&mut app, request).await;

    assert_eq!(status, StatusCode::OK);
    assert!(response.error.is_none());

    let result = response.result.unwrap();
    let content = result["content"].as_array().unwrap();
    assert!(!content.is_empty());

    let text_content = content[0]["text"].as_str().unwrap();
    let materials: Value = serde_json::from_str(text_content).unwrap();

    // Verify materials structure
    assert!(materials.is_array());
    if let Some(first_material) = materials.as_array().unwrap().first() {
        assert!(first_material["id"].is_string());
        assert!(first_material["name"].is_string());
        assert!(first_material["base_price_per_cm3"].is_number());
        assert!(first_material["active"].is_boolean());
    }
}

#[tokio::test]
async fn test_mcp_upload_model_invalid_base64() {
    let pool = setup_test_db().await;
    let temp_dir = TempDir::new().unwrap();
    let config = setup_test_config(&temp_dir);

    let mut app = create_router(pool.clone(), config);

    // Create a session first
    let session_id = create_test_session(&pool).await;

    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(3)),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "upload_model",
            "arguments": {
                "session_id": session_id,
                "filename": "test.stl",
                "file_data": "invalid-base64!!!"
            }
        })),
    };

    let (status, response) = send_mcp_request(&mut app, request).await;

    assert_eq!(status, StatusCode::OK);

    let result = response.result.unwrap();
    let content = result["content"].as_array().unwrap();
    let text = content[0]["text"].as_str().unwrap();

    // Should contain error about invalid base64
    assert!(text.contains("Error") || text.contains("base64"));
    assert_eq!(result["is_error"], json!(true));
}

#[tokio::test]
async fn test_mcp_e2e_quote_workflow() {
    let pool = setup_test_db().await;
    let temp_dir = TempDir::new().unwrap();
    let config = setup_test_config(&temp_dir);

    // Create upload directory
    std::fs::create_dir_all(&config.upload_dir).unwrap();

    let mut app = create_router(pool.clone(), config.clone());

    // Step 1: Create session
    let session_id = create_test_session(&pool).await;

    // Step 2: List materials to get a material ID
    let list_materials_request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "list_materials",
            "arguments": {}
        })),
    };

    let (_, materials_response) = send_mcp_request(&mut app, list_materials_request).await;
    let materials_result = materials_response.result.unwrap();
    let materials_text = materials_result["content"][0]["text"].as_str().unwrap();
    let materials: Value = serde_json::from_str(materials_text).unwrap();
    let material_id = materials[0]["id"].as_str().unwrap().to_string();

    // Step 3: Upload a simple STL model (base64 encoded minimal STL)
    let stl_data = create_minimal_stl();
    let base64_stl = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, stl_data);

    let upload_request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(2)),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "upload_model",
            "arguments": {
                "session_id": session_id,
                "filename": "test_cube.stl",
                "file_data": base64_stl
            }
        })),
    };

    let (status, upload_response) = send_mcp_request(&mut app, upload_request).await;
    assert_eq!(status, StatusCode::OK);

    let upload_result = upload_response.result.unwrap();
    let upload_text = upload_result["content"][0]["text"].as_str().unwrap();
    let upload_data: Value = serde_json::from_str(upload_text).unwrap();
    let model_id = upload_data["model_id"].as_str().unwrap().to_string();

    // Step 4: Configure model with material and quantity
    let configure_request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(3)),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "configure_model",
            "arguments": {
                "session_id": session_id,
                "model_id": model_id,
                "material_id": material_id,
                "quantity": 2
            }
        })),
    };

    let (status, configure_response) = send_mcp_request(&mut app, configure_request).await;
    assert_eq!(status, StatusCode::OK);
    assert!(configure_response.error.is_none());

    let configure_result = configure_response.result.unwrap();
    let configure_text = configure_result["content"][0]["text"].as_str().unwrap();
    let configure_data: Value = serde_json::from_str(configure_text).unwrap();

    assert_eq!(configure_data["model_id"], json!(model_id));
    assert_eq!(configure_data["quantity"], json!(2));
    assert!(configure_data["estimated_price"].is_number());

    // Step 5: Generate quote
    let quote_request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(4)),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "generate_quote",
            "arguments": {
                "session_id": session_id
            }
        })),
    };

    let (status, quote_response) = send_mcp_request(&mut app, quote_request).await;
    assert_eq!(status, StatusCode::OK);
    assert!(quote_response.error.is_none());

    let quote_result = quote_response.result.unwrap();
    let quote_text = quote_result["content"][0]["text"].as_str().unwrap();
    let quote_data: Value = serde_json::from_str(quote_text).unwrap();

    // Verify quote structure
    assert!(quote_data["quote_id"].is_string());
    assert!(quote_data["session_id"].is_string());
    assert!(quote_data["items"].is_array());
    assert!(quote_data["subtotal"].is_number());
    assert!(quote_data["total"].is_number());

    let items = quote_data["items"].as_array().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["model_id"], json!(model_id));
    assert_eq!(items[0]["quantity"], json!(2));
}

#[tokio::test]
async fn test_mcp_configure_nonexistent_model() {
    let pool = setup_test_db().await;
    let temp_dir = TempDir::new().unwrap();
    let config = setup_test_config(&temp_dir);

    let mut app = create_router(pool.clone(), config);

    let session_id = create_test_session(&pool).await;

    let request = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(5)),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "configure_model",
            "arguments": {
                "session_id": session_id,
                "model_id": "nonexistent-model-id",
                "material_id": "some-material-id",
                "quantity": 1
            }
        })),
    };

    let (status, response) = send_mcp_request(&mut app, request).await;
    assert_eq!(status, StatusCode::OK);

    let result = response.result.unwrap();
    let content = result["content"][0]["text"].as_str().unwrap();

    assert!(content.contains("Error"));
    assert!(content.contains("not found") || content.contains("Model"));
    assert_eq!(result["is_error"], json!(true));
}

/// Helper function to create a test session
async fn create_test_session(pool: &PgPool) -> String {
    let session_id = ulid::Ulid::new().to_string();

    sqlx::query(
        "INSERT INTO sessions (id, expires_at) VALUES ($1, NOW() + INTERVAL '24 hours')"
    )
    .bind(&session_id)
    .execute(pool)
    .await
    .unwrap();

    session_id
}

/// Helper function to create minimal valid STL file (binary format)
fn create_minimal_stl() -> Vec<u8> {
    let mut stl = Vec::new();

    // Header (80 bytes)
    stl.extend_from_slice(&[0u8; 80]);

    // Number of triangles (1 triangle)
    stl.extend_from_slice(&1u32.to_le_bytes());

    // Triangle data (50 bytes per triangle)
    // Normal vector
    stl.extend_from_slice(&0f32.to_le_bytes()); // nx
    stl.extend_from_slice(&0f32.to_le_bytes()); // ny
    stl.extend_from_slice(&1f32.to_le_bytes()); // nz

    // Vertex 1
    stl.extend_from_slice(&0f32.to_le_bytes()); // x
    stl.extend_from_slice(&0f32.to_le_bytes()); // y
    stl.extend_from_slice(&0f32.to_le_bytes()); // z

    // Vertex 2
    stl.extend_from_slice(&10f32.to_le_bytes()); // x
    stl.extend_from_slice(&0f32.to_le_bytes());  // y
    stl.extend_from_slice(&0f32.to_le_bytes());  // z

    // Vertex 3
    stl.extend_from_slice(&0f32.to_le_bytes());  // x
    stl.extend_from_slice(&10f32.to_le_bytes()); // y
    stl.extend_from_slice(&0f32.to_le_bytes());  // z

    // Attribute byte count
    stl.extend_from_slice(&0u16.to_le_bytes());

    stl
}

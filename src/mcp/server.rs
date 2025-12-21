//! MCP server implementation with HTTP transport

use axum::{http::StatusCode, Json};
use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;
use std::sync::Arc;

use super::tools;
use super::types::*;

/// MCP Tool definition
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct Tool {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub input_schema: Value,
}

/// Call tool request
#[derive(Debug, Serialize, Deserialize)]
pub struct CallToolRequest {
    pub name: String,
    pub arguments: Value,
}

/// Call tool result
#[derive(Debug, Serialize, Deserialize)]
pub struct CallToolResult {
    pub content: Vec<Content>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
}

/// Content type for tool results
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Content {
    #[serde(rename = "text")]
    Text { text: String },
}

/// List tools result
#[derive(Debug, Serialize, Deserialize)]
pub struct ListToolsResult {
    pub tools: Vec<Tool>,
}

/// MCP Server configuration
#[derive(Clone)]
pub struct McpServerConfig {
    pub pool: PgPool,
    pub upload_dir: String,
    pub max_file_size: usize,
}

/// JSON-RPC 2.0 request
#[derive(Debug, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    pub params: Option<Value>,
}

/// JSON-RPC 2.0 response
#[derive(Debug, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// Handle MCP JSON-RPC request (public API for integration into existing routers)
pub async fn handle_mcp_request_internal(
    config: McpServerConfig,
    request: JsonRpcRequest,
) -> (StatusCode, Json<JsonRpcResponse>) {
    tracing::debug!("MCP request: method={}", request.method);

    let config = Arc::new(config);

    let response = match request.method.as_str() {
        "tools/list" => handle_list_tools(request.id).await,
        "tools/call" => handle_call_tool(config, request.id, request.params).await,
        _ => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: None,
            error: Some(JsonRpcError {
                code: -32601,
                message: format!("Method not found: {}", request.method),
                data: None,
            }),
        },
    };

    (StatusCode::OK, Json(response))
}

/// Handle tools/list request
async fn handle_list_tools(id: Option<Value>) -> JsonRpcResponse {
    let tools = vec![
        Tool {
            name: "upload_model".to_string(),
            description: Some("Upload a 3D model file (STL or 3MF) from base64-encoded data".to_string()),
            input_schema: serde_json::to_value(schema_for!(UploadModelInput)).unwrap(),
        },
        Tool {
            name: "list_materials".to_string(),
            description: Some("List all available printing materials with prices".to_string()),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {},
            }),
        },
        Tool {
            name: "configure_model".to_string(),
            description: Some("Configure a model with material selection and quantity".to_string()),
            input_schema: serde_json::to_value(schema_for!(ConfigureModelInput)).unwrap(),
        },
        Tool {
            name: "generate_quote".to_string(),
            description: Some("Generate a final quote for all configured models in the session".to_string()),
            input_schema: serde_json::to_value(schema_for!(GenerateQuoteInput)).unwrap(),
        },
    ];

    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id,
        result: Some(serde_json::to_value(ListToolsResult { tools }).unwrap()),
        error: None,
    }
}

/// Handle tools/call request
async fn handle_call_tool(
    config: Arc<McpServerConfig>,
    id: Option<Value>,
    params: Option<Value>,
) -> JsonRpcResponse {
    let params = match params {
        Some(p) => p,
        None => {
            return JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32602,
                    message: "Invalid params: params required".to_string(),
                    data: None,
                }),
            };
        }
    };

    let request: CallToolRequest = match serde_json::from_value(params) {
        Ok(r) => r,
        Err(e) => {
            return JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32602,
                    message: format!("Invalid params: {}", e),
                    data: None,
                }),
            };
        }
    };

    let result = match request.name.as_str() {
        "upload_model" => {
            let input: UploadModelInput = match serde_json::from_value(request.arguments) {
                Ok(i) => i,
                Err(e) => {
                    return error_response(id, -32602, format!("Invalid arguments: {}", e));
                }
            };

            match tools::upload_model(
                config.pool.clone(),
                &config.upload_dir,
                config.max_file_size,
                input,
            )
            .await
            {
                Ok(result) => CallToolResult {
                    content: vec![Content::Text {
                        text: serde_json::to_string_pretty(&result).unwrap(),
                    }],
                    is_error: Some(false),
                },
                Err(e) => CallToolResult {
                    content: vec![Content::Text {
                        text: format!("Error: {}", e),
                    }],
                    is_error: Some(true),
                },
            }
        }
        "list_materials" => match tools::list_materials(config.pool.clone()).await {
            Ok(materials) => CallToolResult {
                content: vec![Content::Text {
                    text: serde_json::to_string_pretty(&materials).unwrap(),
                }],
                is_error: Some(false),
            },
            Err(e) => CallToolResult {
                content: vec![Content::Text {
                    text: format!("Error: {}", e),
                }],
                is_error: Some(true),
            },
        },
        "configure_model" => {
            let input: ConfigureModelInput = match serde_json::from_value(request.arguments) {
                Ok(i) => i,
                Err(e) => {
                    return error_response(id, -32602, format!("Invalid arguments: {}", e));
                }
            };

            match tools::configure_model(config.pool.clone(), input).await {
                Ok(result) => CallToolResult {
                    content: vec![Content::Text {
                        text: serde_json::to_string_pretty(&result).unwrap(),
                    }],
                    is_error: Some(false),
                },
                Err(e) => CallToolResult {
                    content: vec![Content::Text {
                        text: format!("Error: {}", e),
                    }],
                    is_error: Some(true),
                },
            }
        }
        "generate_quote" => {
            let input: GenerateQuoteInput = match serde_json::from_value(request.arguments) {
                Ok(i) => i,
                Err(e) => {
                    return error_response(id, -32602, format!("Invalid arguments: {}", e));
                }
            };

            match tools::generate_quote(config.pool.clone(), input).await {
                Ok(result) => CallToolResult {
                    content: vec![Content::Text {
                        text: serde_json::to_string_pretty(&result).unwrap(),
                    }],
                    is_error: Some(false),
                },
                Err(e) => CallToolResult {
                    content: vec![Content::Text {
                        text: format!("Error: {}", e),
                    }],
                    is_error: Some(true),
                },
            }
        }
        _ => {
            return error_response(id, -32601, format!("Unknown tool: {}", request.name));
        }
    };

    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id,
        result: Some(serde_json::to_value(result).unwrap()),
        error: None,
    }
}

fn error_response(id: Option<Value>, code: i32, message: String) -> JsonRpcResponse {
    JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        id,
        result: None,
        error: Some(JsonRpcError {
            code,
            message,
            data: None,
        }),
    }
}

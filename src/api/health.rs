//! Health check endpoints for Kubernetes-style liveness and readiness probes
//!
//! This module provides two health check endpoints:
//! - /health: Liveness probe (is the service up?)
//! - /ready: Readiness probe (can the service handle requests?)

use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::path::Path;

use crate::api::routes::AppState;

/// Health check response
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    /// Overall status
    pub status: String,
    /// Individual component checks (only for readiness)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checks: Option<HealthChecks>,
}

/// Individual health checks
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthChecks {
    /// Database connection check
    pub database: ComponentHealth,
    /// Filesystem check
    pub filesystem: ComponentHealth,
}

/// Health status for individual component
#[derive(Debug, Serialize, Deserialize)]
pub struct ComponentHealth {
    /// Status: "ok" or "error"
    pub status: String,
    /// Optional error message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Liveness probe endpoint
///
/// Returns 200 OK if the service is running.
/// This endpoint should never fail unless the service is completely down.
///
/// # Endpoint
/// GET /health
///
/// # Response
/// ```json
/// {
///   "status": "ok"
/// }
/// ```
pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        checks: None,
    })
}

/// Readiness probe endpoint
///
/// Returns 200 OK if the service is ready to handle requests.
/// Checks:
/// - Database connection (can execute queries)
/// - Filesystem access (can write to upload directory)
///
/// Returns 503 Service Unavailable if any check fails.
///
/// # Endpoint
/// GET /ready
///
/// # Response
/// ```json
/// {
///   "status": "ready",
///   "checks": {
///     "database": {"status": "ok"},
///     "filesystem": {"status": "ok"}
///   }
/// }
/// ```
///
/// Or on failure:
/// ```json
/// {
///   "status": "not_ready",
///   "checks": {
///     "database": {"status": "error", "message": "connection failed"},
///     "filesystem": {"status": "ok"}
///   }
/// }
/// ```
pub async fn ready(State(state): State<AppState>) -> (StatusCode, Json<HealthResponse>) {
    // Check database connection
    let db_check = check_database(&state.pool).await;

    // Check filesystem access
    let fs_check = check_filesystem().await;

    // Determine overall status
    let is_ready = db_check.status == "ok" && fs_check.status == "ok";
    let status_code = if is_ready {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    let response = HealthResponse {
        status: if is_ready {
            "ready".to_string()
        } else {
            "not_ready".to_string()
        },
        checks: Some(HealthChecks {
            database: db_check,
            filesystem: fs_check,
        }),
    };

    (status_code, Json(response))
}

/// Check database connectivity
async fn check_database(pool: &PgPool) -> ComponentHealth {
    match sqlx::query("SELECT 1").execute(pool).await {
        Ok(_) => ComponentHealth {
            status: "ok".to_string(),
            message: None,
        },
        Err(e) => ComponentHealth {
            status: "error".to_string(),
            message: Some(format!("Database connection failed: {}", e)),
        },
    }
}

/// Check filesystem access
async fn check_filesystem() -> ComponentHealth {
    let upload_dir = std::env::var("UPLOAD_DIR").unwrap_or_else(|_| "./uploads".to_string());
    let test_file = format!("{}/.health_check", upload_dir);

    // Try to write a test file
    match tokio::fs::write(&test_file, "health_check").await {
        Ok(_) => {
            // Clean up test file
            let _ = tokio::fs::remove_file(&test_file).await;

            ComponentHealth {
                status: "ok".to_string(),
                message: None,
            }
        }
        Err(e) => {
            // Check if directory exists
            if !Path::new(&upload_dir).exists() {
                ComponentHealth {
                    status: "error".to_string(),
                    message: Some(format!("Upload directory does not exist: {}", upload_dir)),
                }
            } else {
                ComponentHealth {
                    status: "error".to_string(),
                    message: Some(format!("Cannot write to upload directory: {}", e)),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_always_ok() {
        let response = health().await;
        assert_eq!(response.0.status, "ok");
        assert!(response.0.checks.is_none());
    }

    #[tokio::test]
    async fn test_filesystem_check_fails_on_nonexistent_dir() {
        // Set upload dir to a directory that doesn't exist
        // SAFETY: This is safe in tests as we're not reading from other threads
        unsafe {
            std::env::set_var("UPLOAD_DIR", "/nonexistent/directory");
        }

        let result = check_filesystem().await;
        assert_eq!(result.status, "error");
        assert!(result.message.is_some());
    }
}

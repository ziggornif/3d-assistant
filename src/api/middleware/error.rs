use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Application error types
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Invalid file format: {0}")]
    InvalidFileFormat(String),

    #[error("File too large: {0} bytes (max: {1} bytes)")]
    FileTooLarge(i64, i64),

    #[error("Session not found: {0}")]
    SessionNotFound(String),

    #[error("Session expired: {0}")]
    SessionExpired(String),

    #[error("Model not found: {0}")]
    ModelNotFound(String),

    #[error("Material not found: {0}")]
    MaterialNotFound(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("File processing error: {0}")]
    FileProcessing(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Internal server error: {0}")]
    Internal(String),
}

/// Error response structure
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: ErrorDetail,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorDetail {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_response) = match &self {
            AppError::InvalidFileFormat(format) => (
                StatusCode::BAD_REQUEST,
                ErrorResponse {
                    error: ErrorDetail {
                        code: "INVALID_FILE_FORMAT".to_string(),
                        message: "Seuls les fichiers STL et 3MF sont acceptés".to_string(),
                        details: Some(serde_json::json!({
                            "received": format,
                            "accepted": ["stl", "3mf"]
                        })),
                    },
                },
            ),
            AppError::FileTooLarge(size, max) => (
                StatusCode::PAYLOAD_TOO_LARGE,
                ErrorResponse {
                    error: ErrorDetail {
                        code: "FILE_TOO_LARGE".to_string(),
                        message: format!(
                            "Le fichier est trop volumineux (max: {} MB)",
                            max / 1_000_000
                        ),
                        details: Some(serde_json::json!({
                            "size_bytes": size,
                            "max_bytes": max
                        })),
                    },
                },
            ),
            AppError::SessionNotFound(id) => (
                StatusCode::NOT_FOUND,
                ErrorResponse {
                    error: ErrorDetail {
                        code: "SESSION_NOT_FOUND".to_string(),
                        message: "Session non trouvée".to_string(),
                        details: Some(serde_json::json!({ "session_id": id })),
                    },
                },
            ),
            AppError::SessionExpired(id) => (
                StatusCode::GONE,
                ErrorResponse {
                    error: ErrorDetail {
                        code: "SESSION_EXPIRED".to_string(),
                        message: "La session a expiré".to_string(),
                        details: Some(serde_json::json!({ "session_id": id })),
                    },
                },
            ),
            AppError::ModelNotFound(id) => (
                StatusCode::NOT_FOUND,
                ErrorResponse {
                    error: ErrorDetail {
                        code: "MODEL_NOT_FOUND".to_string(),
                        message: "Modèle non trouvé".to_string(),
                        details: Some(serde_json::json!({ "model_id": id })),
                    },
                },
            ),
            AppError::MaterialNotFound(id) => (
                StatusCode::NOT_FOUND,
                ErrorResponse {
                    error: ErrorDetail {
                        code: "MATERIAL_NOT_FOUND".to_string(),
                        message: "Matériau non trouvé".to_string(),
                        details: Some(serde_json::json!({ "material_id": id })),
                    },
                },
            ),
            AppError::Database(e) => {
                tracing::error!("Database error: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ErrorResponse {
                        error: ErrorDetail {
                            code: "DATABASE_ERROR".to_string(),
                            message: "Erreur de base de données".to_string(),
                            details: None,
                        },
                    },
                )
            }
            AppError::FileProcessing(msg) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                ErrorResponse {
                    error: ErrorDetail {
                        code: "FILE_PROCESSING_ERROR".to_string(),
                        message: msg.clone(),
                        details: None,
                    },
                },
            ),
            _ => {
                tracing::error!("Internal error: {}", self);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ErrorResponse {
                        error: ErrorDetail {
                            code: "INTERNAL_ERROR".to_string(),
                            message: "Erreur interne du serveur".to_string(),
                            details: None,
                        },
                    },
                )
            }
        };

        (status, Json(error_response)).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;

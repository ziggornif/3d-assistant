use axum::{
    Json,
    extract::{Multipart, Path, State},
    http::StatusCode,
};
use axum_extra::extract::cookie::CookieJar;
use chrono::NaiveDateTime;
use serde::Serialize;
use std::path::PathBuf;

use crate::models::quote::UploadedModel;
use crate::{
    api::middleware::{AppError, AppResult, sanitize_filename},
    persistence::models::create,
};
use crate::{api::routes::AppState, persistence::models};
use crate::{
    business::{AuthService, SessionService, file_processor},
    models::model::CreateModel,
};

#[derive(Serialize)]
pub struct CreateSessionResponse {
    pub session_id: String,
    pub created_at: NaiveDateTime,
    pub expires_at: NaiveDateTime,
}

/// Create a new quote session
/// If the user is authenticated, creates an authenticated session (30 days).
/// Otherwise, creates an anonymous demo session (24 hours).
pub async fn create_session(
    State(state): State<AppState>,
    jar: CookieJar,
) -> AppResult<Json<CreateSessionResponse>> {
    // Check if user is authenticated
    let authenticated_user = if let Some(cookie) = jar.get("user_session") {
        let auth_service = AuthService::new(state.pool.clone());
        auth_service.verify_session(cookie.value()).await.ok()
    } else {
        None
    };

    let session_service = SessionService::new(state.pool.clone(), &state.config.upload_dir);

    let session = if let Some(user) = authenticated_user {
        session_service.create_authenticated_session(&user.id).await?
    } else {
        session_service.create_session().await?
    };

    tracing::info!(
        "Created new {} session: {}",
        session.session_type,
        session.id
    );

    Ok(Json(CreateSessionResponse {
        session_id: session.id,
        created_at: session.created_at,
        expires_at: session.expires_at,
    }))
}

#[derive(Serialize)]
pub struct UploadModelResponse {
    pub model_id: String,
    pub filename: String,
    pub volume_cm3: f64,
    pub dimensions_mm: DimensionsResponse,
    pub triangle_count: i64,
    pub file_size_bytes: i64,
    pub preview_url: String,
}

#[derive(Serialize)]
pub struct DimensionsResponse {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// Upload a 3D model file
pub async fn upload_model(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
    mut multipart: Multipart,
) -> AppResult<Json<UploadModelResponse>> {
    // Verify session exists and is valid
    let session_service = SessionService::new(state.pool.clone(), &state.config.upload_dir);
    session_service.get_session(&session_id).await?;

    // Process multipart upload
    let mut file_data: Option<(String, Vec<u8>)> = None;

    while let Some(mut field) = multipart.next_field().await.map_err(|e| {
        tracing::error!("Failed to get next field: {:?}", e);
        AppError::FileProcessing(format!("Erreur multipart: {}", e))
    })? {
        let field_name = field.name().unwrap_or("unknown").to_string();
        let file_name_opt = field.file_name().map(|s| s.to_string());
        let content_type = field.content_type().map(|s| s.to_string());

        tracing::debug!(
            "Processing field: name={}, filename={:?}, content_type={:?}",
            field_name,
            file_name_opt,
            content_type
        );

        if field_name == "file" {
            let raw_filename = file_name_opt
                .ok_or_else(|| AppError::FileProcessing("Nom de fichier manquant".to_string()))?;

            // Validate MIME type for security (prevent executable uploads)
            if let Some(ref mime) = content_type {
                let valid_mimes = [
                    "application/sla",
                    "model/stl",
                    "application/vnd.ms-3mfdocument",
                    "application/x-3mf",
                    "application/octet-stream", // Generic binary, common for 3D files
                    "model/3mf",
                ];
                if !valid_mimes.iter().any(|&valid| mime.contains(valid)) {
                    tracing::warn!("Invalid MIME type rejected: {}", mime);
                    return Err(AppError::InvalidFileFormat(mime.clone()));
                }
            }

            // Sanitize filename to prevent directory traversal attacks
            let filename = sanitize_filename(&raw_filename);

            tracing::info!("Reading bytes for file: {}", filename);

            // Read bytes chunk by chunk to handle browser streams
            let mut all_bytes = Vec::new();

            loop {
                match field.chunk().await {
                    Ok(Some(chunk)) => {
                        all_bytes.extend_from_slice(&chunk);
                    }
                    Ok(None) => {
                        // End of stream
                        break;
                    }
                    Err(e) => {
                        tracing::error!("Failed to read chunk: {:?}", e);
                        return Err(AppError::FileProcessing(format!("Erreur lecture: {}", e)));
                    }
                }
            }

            tracing::info!("Successfully read {} bytes total", all_bytes.len());

            file_data = Some((filename, all_bytes));
            break;
        }
    }

    let (filename, bytes) =
        file_data.ok_or_else(|| AppError::FileProcessing("Aucun fichier trouvé".to_string()))?;

    // Validate file
    let file_format =
        file_processor::validate_file(&bytes, &filename, state.config.max_file_size_bytes)?;

    tracing::info!(
        "Received file: {} ({} bytes, format: {})",
        filename,
        bytes.len(),
        file_format
    );

    // Create model record FIRST to get the ID
    let mut model = UploadedModel::new(
        session_id.clone(),
        filename.clone(),
        file_format.clone(),
        bytes.len() as i64,
        String::new(), // Will update after saving
    );

    // Save file to disk using the model's ID
    let file_path = PathBuf::from(&state.config.upload_dir)
        .join(&session_id)
        .join(format!("{}.{}", model.id, file_format));

    // Create session directory if needed
    if let Some(parent) = file_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| AppError::Internal(format!("Erreur création dossier: {}", e)))?;
    }

    std::fs::write(&file_path, &bytes)
        .map_err(|e| AppError::Internal(format!("Erreur sauvegarde: {}", e)))?;

    // Update model with actual file path
    model.file_path = file_path.to_string_lossy().to_string();

    // Process file based on format
    let file_path_str = file_path
        .to_str()
        .ok_or_else(|| AppError::Internal("Chemin invalide".to_string()))?;

    let processed = match file_format.as_str() {
        "stl" => file_processor::process_stl_file(file_path_str)?,
        "3mf" => file_processor::process_3mf_file(file_path_str)?,
        _ => {
            return Err(AppError::FileProcessing(format!(
                "Format non supporté pour le traitement: {}",
                file_format
            )));
        }
    };

    model.volume_cm3 = Some(processed.volume_cm3);
    model.set_dimensions(processed.dimensions_mm);
    model.triangle_count = Some(processed.triangle_count);
    model.set_support_analysis(processed.support_analysis.clone());

    // Save to database
    create(
        &state.pool,
        CreateModel {
            id: &model.id,
            session_id: &model.session_id,
            filename: &model.filename,
            file_format: &model.file_format,
            file_size_bytes: model.file_size_bytes,
            volume_cm3: model.volume_cm3,
            dimensions_mm: model.dimensions_mm.as_deref(),
            triangle_count: model.triangle_count,
            material_id: model.material_id.as_deref(),
            file_path: &model.file_path,
            preview_url: &model.preview_url,
            created_at: model.created_at,
            support_analysis: model.support_analysis.as_deref(),
        },
    )
    .await?;

    tracing::info!(
        "Uploaded model {} for session {}: volume={:.2}cm³",
        model.id,
        session_id,
        processed.volume_cm3
    );

    let dims = model
        .get_dimensions()
        .unwrap_or(crate::models::quote::Dimensions {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        });

    Ok(Json(UploadModelResponse {
        model_id: model.id.clone(),
        filename,
        volume_cm3: processed.volume_cm3,
        dimensions_mm: DimensionsResponse {
            x: dims.x,
            y: dims.y,
            z: dims.z,
        },
        triangle_count: processed.triangle_count,
        file_size_bytes: model.file_size_bytes,
        preview_url: model.preview_url,
    }))
}

pub async fn get_session_models(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> AppResult<Json<Vec<UploadedModel>>> {
    let session_service = SessionService::new(state.pool.clone(), &state.config.upload_dir);
    session_service.get_session(&session_id).await?;

    let models = models::find_by_session(&state.pool, &session_id).await?;

    Ok(Json(models))
}

/// Delete an uploaded model
pub async fn delete_model(
    State(state): State<AppState>,
    Path((session_id, model_id)): Path<(String, String)>,
) -> AppResult<StatusCode> {
    // Verify session
    let session_service = SessionService::new(state.pool.clone(), &state.config.upload_dir);
    session_service.get_session(&session_id).await?;

    // Get model to find file path
    let model = models::find_by_id_and_session(&state.pool, &model_id, &session_id).await?;

    let model = model.ok_or_else(|| AppError::ModelNotFound(model_id.clone()))?;

    // Delete file from disk
    if let Err(e) = std::fs::remove_file(&model.file_path) {
        tracing::warn!("Failed to delete file {}: {}", model.file_path, e);
    }

    // Delete from database
    models::delete(&state.pool, &model_id).await?;

    tracing::info!("Deleted model {} from session {}", model_id, session_id);

    Ok(StatusCode::NO_CONTENT)
}

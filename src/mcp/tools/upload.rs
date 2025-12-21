//! Upload model tool implementation

use crate::{
    api::middleware::sanitize_filename,
    business::{file_processor, SessionService},
    mcp::types::{Dimensions, UploadModelInput, UploadModelResult},
    models::{model::CreateModel, quote::UploadedModel},
};
use base64::{engine::general_purpose, Engine as _};
use sqlx::PgPool;
use std::path::PathBuf;

/// Upload a 3D model file from base64-encoded data
pub async fn upload_model(
    pool: PgPool,
    upload_dir: &str,
    max_file_size: usize,
    input: UploadModelInput,
) -> Result<UploadModelResult, String> {
    // Verify session exists
    let session_service = SessionService::new(pool.clone(), upload_dir);
    session_service
        .get_session(&input.session_id)
        .await
        .map_err(|e| format!("Session not found: {}", e))?;

    // Decode base64 file data
    let bytes = general_purpose::STANDARD
        .decode(&input.file_data)
        .map_err(|e| format!("Invalid base64 encoding: {}", e))?;

    // Check file size
    if bytes.len() > max_file_size {
        return Err(format!(
            "File size {} exceeds maximum of {} bytes",
            bytes.len(),
            max_file_size
        ));
    }

    // Sanitize filename
    let filename = sanitize_filename(&input.filename);

    // Validate file format
    let file_format = file_processor::validate_file(&bytes, &filename, max_file_size as i64)
        .map_err(|e| format!("File validation failed: {}", e))?;

    tracing::info!(
        "MCP: Received file: {} ({} bytes, format: {})",
        filename,
        bytes.len(),
        file_format
    );

    // Create model record
    let mut model = UploadedModel::new(
        input.session_id.clone(),
        filename.clone(),
        file_format.clone(),
        bytes.len() as i64,
        String::new(), // Will update after saving
    );

    // Save file to disk
    let file_path = PathBuf::from(upload_dir)
        .join(&input.session_id)
        .join(format!("{}.{}", model.id, file_format));

    // Create session directory if needed
    if let Some(parent) = file_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory: {}", e))?;
    }

    std::fs::write(&file_path, &bytes)
        .map_err(|e| format!("Failed to save file: {}", e))?;

    // Update model with file path
    model.file_path = file_path.to_string_lossy().to_string();

    // Process file to extract metadata
    let file_path_str = file_path
        .to_str()
        .ok_or_else(|| "Invalid file path".to_string())?;

    let processed = match file_format.as_str() {
        "stl" => file_processor::process_stl_file(file_path_str)
            .map_err(|e| format!("STL processing failed: {}", e))?,
        "3mf" => file_processor::process_3mf_file(file_path_str)
            .map_err(|e| format!("3MF processing failed: {}", e))?,
        _ => return Err(format!("Unsupported file format: {}", file_format)),
    };

    model.volume_cm3 = Some(processed.volume_cm3);
    model.set_dimensions(processed.dimensions_mm);
    model.triangle_count = Some(processed.triangle_count);

    // Save model to database
    crate::persistence::models::create(
        &pool,
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
    .await
    .map_err(|e| format!("Failed to save model: {}", e))?;

    tracing::info!("MCP: Created model {} for session {}", model.id, input.session_id);

    // Build response - extract dimensions from JSON
    let dimensions = if let Some(dims_json) = model.dimensions_mm {
        if let Ok(dims) = serde_json::from_str::<serde_json::Value>(&dims_json) {
            Some(Dimensions {
                x: dims["x"].as_f64().unwrap_or(0.0),
                y: dims["y"].as_f64().unwrap_or(0.0),
                z: dims["z"].as_f64().unwrap_or(0.0),
            })
        } else {
            None
        }
    } else {
        None
    };

    Ok(UploadModelResult {
        model_id: model.id,
        filename: model.filename,
        file_format: model.file_format,
        volume_cm3: model.volume_cm3,
        dimensions_mm: dimensions,
        triangle_count: model.triangle_count.map(|t| t as i32),
    })
}

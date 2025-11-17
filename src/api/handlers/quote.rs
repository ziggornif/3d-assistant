use axum::{
    extract::{Path, State},
    Json,
};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::api::middleware::{AppError, AppResult};
use crate::api::routes::AppState;
use crate::models::material::Material;
use crate::models::quote::UploadedModel;
use crate::services::SessionService;

#[derive(Deserialize)]
pub struct ConfigureModelRequest {
    pub material_id: String,
}

#[derive(Serialize)]
pub struct ConfigureModelResponse {
    pub model_id: String,
    pub material_id: String,
    pub estimated_price: f64,
}

/// Configure a model with material selection
pub async fn configure_model(
    State(state): State<AppState>,
    Path((session_id, model_id)): Path<(String, String)>,
    Json(body): Json<ConfigureModelRequest>,
) -> AppResult<Json<ConfigureModelResponse>> {
    // Verify session exists
    let session_service = SessionService::new(state.pool.clone(), &state.config.upload_dir);
    session_service.get_session(&session_id).await?;

    // Fetch the model
    let model: Option<UploadedModel> = sqlx::query_as(
        r#"
        SELECT id, session_id, filename, file_format, file_size_bytes, volume_cm3,
               dimensions_mm, triangle_count, material_id, file_path, created_at
        FROM uploaded_models
        WHERE id = ? AND session_id = ?
        "#,
    )
    .bind(&model_id)
    .bind(&session_id)
    .fetch_optional(&state.pool)
    .await?;

    let model = model.ok_or_else(|| AppError::ModelNotFound(model_id.clone()))?;

    // Fetch the material
    let material: Option<Material> = sqlx::query_as(
        r#"
        SELECT id, service_type_id, name, description, price_per_cm3,
               color, properties, active, created_at, updated_at
        FROM materials
        WHERE id = ? AND active = 1
        "#,
    )
    .bind(&body.material_id)
    .fetch_optional(&state.pool)
    .await?;

    let material = material.ok_or_else(|| AppError::MaterialNotFound(body.material_id.clone()))?;

    // Update model with material_id
    sqlx::query("UPDATE uploaded_models SET material_id = ? WHERE id = ?")
        .bind(&body.material_id)
        .bind(&model_id)
        .execute(&state.pool)
        .await?;

    // Calculate estimated price
    let volume = model.volume_cm3.unwrap_or(0.0);
    let estimated_price = material.calculate_price(volume);

    tracing::info!(
        "Configured model {} with material {} ({}), estimated price: {}€",
        model_id,
        material.name,
        body.material_id,
        estimated_price
    );

    Ok(Json(ConfigureModelResponse {
        model_id,
        material_id: body.material_id,
        estimated_price: estimated_price.to_string().parse::<f64>().unwrap_or(0.0),
    }))
}

#[derive(Serialize)]
pub struct QuoteResponse {
    pub quote_id: String,
    pub items: Vec<QuoteItemResponse>,
    pub subtotal: f64,
    pub fees: f64,
    pub total: f64,
    pub minimum_applied: bool,
    pub calculated_total: f64,
    pub breakdown: serde_json::Value,
}

#[derive(Serialize)]
pub struct QuoteItemResponse {
    pub model_id: String,
    pub model_name: String,
    pub material_name: String,
    pub volume_cm3: f64,
    pub price: f64,
}

/// Generate a final quote
pub async fn generate_quote(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> AppResult<Json<QuoteResponse>> {
    // Verify session exists
    let session_service = SessionService::new(state.pool.clone(), &state.config.upload_dir);
    session_service.get_session(&session_id).await?;

    // Get all models with their materials
    let models: Vec<UploadedModel> = sqlx::query_as(
        r#"
        SELECT id, session_id, filename, file_format, file_size_bytes, volume_cm3,
               dimensions_mm, triangle_count, material_id, file_path, created_at
        FROM uploaded_models
        WHERE session_id = ?
        "#,
    )
    .bind(&session_id)
    .fetch_all(&state.pool)
    .await?;

    if models.is_empty() {
        return Err(AppError::Internal("No models in session".to_string()));
    }

    // Build quote items
    let mut items = Vec::new();
    for model in &models {
        let material_id = model.material_id.as_ref().ok_or_else(|| {
            AppError::Internal(format!("Model {} has no material assigned", model.id))
        })?;

        let material: Material = sqlx::query_as(
            r#"
            SELECT id, service_type_id, name, description, price_per_cm3,
                   color, properties, active, created_at, updated_at
            FROM materials
            WHERE id = ?
            "#,
        )
        .bind(material_id)
        .fetch_one(&state.pool)
        .await?;

        let volume = model.volume_cm3.unwrap_or(0.0);
        let material_cost =
            crate::services::pricing::calculate_model_price(volume, material.price_per_cm3);

        items.push(crate::services::pricing::QuoteItem {
            model_id: model.id.clone(),
            model_name: model.filename.clone(),
            material_id: material.id.clone(),
            material_name: material.name.clone(),
            volume_cm3: volume,
            price_per_cm3: material.price_per_cm3,
            material_cost,
        });
    }

    // Generate breakdown
    let breakdown = crate::services::pricing::generate_quote_breakdown(items.clone());

    // Create quote record
    let quote_id = Ulid::new().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let breakdown_json = serde_json::to_string(&breakdown)
        .map_err(|e| AppError::Internal(format!("JSON serialization error: {}", e)))?;

    sqlx::query(
        r#"
        INSERT INTO quotes (id, session_id, total_price, breakdown, status, created_at)
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&quote_id)
    .bind(&session_id)
    .bind(breakdown.total)
    .bind(&breakdown_json)
    .bind("generated")
    .bind(&now)
    .execute(&state.pool)
    .await?;

    let response_items: Vec<QuoteItemResponse> = items
        .iter()
        .map(|item| QuoteItemResponse {
            model_id: item.model_id.clone(),
            model_name: item.model_name.clone(),
            material_name: item.material_name.clone(),
            volume_cm3: item.volume_cm3,
            price: item.material_cost,
        })
        .collect();

    tracing::info!(
        "Generated quote {} for session {}: total={}€",
        quote_id,
        session_id,
        breakdown.total
    );

    Ok(Json(QuoteResponse {
        quote_id,
        items: response_items,
        subtotal: breakdown.subtotal,
        fees: breakdown.base_fee,
        total: breakdown.total,
        minimum_applied: breakdown.minimum_applied,
        calculated_total: breakdown.calculated_total,
        breakdown: serde_json::json!({
            "base_fee": breakdown.base_fee,
            "subtotal": breakdown.subtotal,
            "total": breakdown.total,
            "minimum_applied": breakdown.minimum_applied,
            "calculated_total": breakdown.calculated_total,
        }),
    }))
}

/// Get current quote calculation (without persisting)
pub async fn get_current_quote(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> AppResult<Json<QuoteResponse>> {
    // Verify session exists
    let session_service = SessionService::new(state.pool.clone(), &state.config.upload_dir);
    session_service.get_session(&session_id).await?;

    // Get all models with their materials
    let models: Vec<UploadedModel> = sqlx::query_as(
        r#"
        SELECT id, session_id, filename, file_format, file_size_bytes, volume_cm3,
               dimensions_mm, triangle_count, material_id, file_path, created_at
        FROM uploaded_models
        WHERE session_id = ?
        "#,
    )
    .bind(&session_id)
    .fetch_all(&state.pool)
    .await?;

    // Build quote items (only for models with materials assigned)
    let mut items = Vec::new();
    for model in &models {
        if let Some(material_id) = &model.material_id {
            let material: Option<Material> = sqlx::query_as(
                r#"
                SELECT id, service_type_id, name, description, price_per_cm3,
                       color, properties, active, created_at, updated_at
                FROM materials
                WHERE id = ?
                "#,
            )
            .bind(material_id)
            .fetch_optional(&state.pool)
            .await?;

            if let Some(material) = material {
                let volume = model.volume_cm3.unwrap_or(0.0);
                let material_cost =
                    crate::services::pricing::calculate_model_price(volume, material.price_per_cm3);

                items.push(crate::services::pricing::QuoteItem {
                    model_id: model.id.clone(),
                    model_name: model.filename.clone(),
                    material_id: material.id.clone(),
                    material_name: material.name.clone(),
                    volume_cm3: volume,
                    price_per_cm3: material.price_per_cm3,
                    material_cost,
                });
            }
        }
    }

    // Generate breakdown
    let breakdown = crate::services::pricing::generate_quote_breakdown(items.clone());

    let response_items: Vec<QuoteItemResponse> = items
        .iter()
        .map(|item| QuoteItemResponse {
            model_id: item.model_id.clone(),
            model_name: item.model_name.clone(),
            material_name: item.material_name.clone(),
            volume_cm3: item.volume_cm3,
            price: item.material_cost,
        })
        .collect();

    Ok(Json(QuoteResponse {
        quote_id: String::new(), // No quote ID for preview
        items: response_items,
        subtotal: breakdown.subtotal,
        fees: breakdown.base_fee,
        total: breakdown.total,
        minimum_applied: breakdown.minimum_applied,
        calculated_total: breakdown.calculated_total,
        breakdown: serde_json::json!({
            "base_fee": breakdown.base_fee,
            "subtotal": breakdown.subtotal,
            "total": breakdown.total,
            "minimum_applied": breakdown.minimum_applied,
            "calculated_total": breakdown.calculated_total,
        }),
    }))
}

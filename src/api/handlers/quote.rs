use axum::{
    Json,
    extract::{Path, State},
};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::api::routes::AppState;
use crate::business::SessionService;
use crate::{
    api::middleware::{AppError, AppResult},
    persistence::materials,
    persistence::models,
    persistence::quotes,
};

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
    let model = models::find_by_id_and_session(&state.pool, &model_id, &session_id).await?;

    let model = model.ok_or_else(|| AppError::ModelNotFound(model_id.clone()))?;

    // Fetch the material
    let material = materials::find_by_id(&state.pool, &body.material_id).await?;

    let material = material.ok_or_else(|| AppError::MaterialNotFound(body.material_id.clone()))?;

    // Update model with material_id
    models::update_material(&state.pool, &model_id, &body.material_id).await?;

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
    let session = session_service.get_session(&session_id).await?;

    // Block quote generation for anonymous sessions (demo mode)
    if session.is_anonymous() {
        return Err(AppError::Forbidden(
            "Inscrivez-vous pour generer un devis".to_string(),
        ));
    }

    // Get all models with their materials
    let models = models::find_by_session(&state.pool, &session_id).await?;
    if models.is_empty() {
        return Err(AppError::Internal("No models in session".to_string()));
    }

    // Build quote items
    let mut items = Vec::new();
    for model in &models {
        let material_id = model.material_id.as_ref().ok_or_else(|| {
            AppError::Internal(format!("Model {} has no material assigned", model.id))
        })?;

        let material = materials::find_by_id(&state.pool, material_id).await?;
        if let Some(material) = material {
            let base_volume = model.volume_cm3.unwrap_or(0.0);

            // Calculate support volume based on support analysis
            let support_percentage = model
                .get_support_analysis()
                .map_or(0.0, |s| f64::from(s.estimated_support_material_percentage));

            let support_volume = base_volume * (support_percentage / 100.0);
            let total_volume = base_volume + support_volume;

            let material_cost = crate::business::pricing::calculate_model_price(
                total_volume,
                material.price_per_cm3,
            );

            items.push(crate::business::pricing::QuoteItem {
                model_id: model.id.clone(),
                model_name: model.filename.clone(),
                material_id: material.id.clone(),
                material_name: material.name.clone(),
                volume_cm3: total_volume,
                price_per_cm3: material.price_per_cm3,
                material_cost,
            });
        }
    }

    // Generate breakdown
    let breakdown = crate::business::pricing::generate_quote_breakdown(items.clone());

    // Create quote record
    let quote_id = Ulid::new().to_string();
    let now = chrono::Utc::now().naive_utc();
    let breakdown_json = serde_json::to_string(&breakdown)
        .map_err(|e| AppError::Internal(format!("JSON serialization error: {e}")))?;

    quotes::create(
        &state.pool,
        &quote_id,
        &session_id,
        breakdown.total,
        &breakdown_json,
        "generated",
        now,
    )
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
    let models = models::find_by_session(&state.pool, &session_id).await?;

    // Build quote items (only for models with materials assigned)
    let mut items = Vec::new();
    for model in &models {
        if let Some(material_id) = &model.material_id {
            let material = materials::find_by_id(&state.pool, material_id).await?;

            if let Some(material) = material {
                let base_volume = model.volume_cm3.unwrap_or(0.0);

                // Calculate support volume based on support analysis
                let support_percentage = model
                    .get_support_analysis()
                    .map_or(0.0, |s| f64::from(s.estimated_support_material_percentage));

                let support_volume = base_volume * (support_percentage / 100.0);
                let total_volume = base_volume + support_volume;

                let material_cost = crate::business::pricing::calculate_model_price(
                    total_volume,
                    material.price_per_cm3,
                );

                items.push(crate::business::pricing::QuoteItem {
                    model_id: model.id.clone(),
                    model_name: model.filename.clone(),
                    material_id: material.id.clone(),
                    material_name: material.name.clone(),
                    volume_cm3: total_volume,
                    price_per_cm3: material.price_per_cm3,
                    material_cost,
                });
            }
        }
    }

    // Generate breakdown
    let breakdown = crate::business::pricing::generate_quote_breakdown(items.clone());

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

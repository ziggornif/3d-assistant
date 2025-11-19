use axum::{
    Json,
    extract::{Path, State},
};
use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::api::middleware::{AppError, AppResult};
use crate::api::routes::AppState;
use crate::business::{CleanupResult, SessionService};
use crate::models::material::Material;
use crate::persistence;

#[derive(Serialize)]
pub struct AdminMaterialResponse {
    pub id: String,
    pub service_type_id: String,
    pub name: String,
    pub description: Option<String>,
    pub price_per_cm3: f64,
    pub color: Option<String>,
    pub properties: Option<serde_json::Value>,
    pub active: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl From<Material> for AdminMaterialResponse {
    fn from(m: Material) -> Self {
        Self {
            id: m.id,
            service_type_id: m.service_type_id,
            name: m.name,
            description: m.description,
            price_per_cm3: m.price_per_cm3,
            color: m.color,
            properties: m.properties.and_then(|s| serde_json::from_str(&s).ok()),
            active: m.active,
            created_at: m.created_at,
            updated_at: m.updated_at,
        }
    }
}

/// List all materials (admin view - includes inactive)
pub async fn list_materials(
    State(state): State<AppState>,
) -> AppResult<Json<Vec<AdminMaterialResponse>>> {
    let materials = persistence::materials::list_all(&state.pool).await?;

    let responses: Vec<AdminMaterialResponse> = materials.into_iter().map(Into::into).collect();
    tracing::info!("Admin listed {} materials", responses.len());
    Ok(Json(responses))
}

#[derive(Deserialize)]
pub struct CreateMaterialRequest {
    pub name: String,
    pub service_type_id: String,
    pub description: Option<String>,
    pub price_per_cm3: f64,
    pub color: Option<String>,
    pub properties: Option<serde_json::Value>,
}

/// Create a new material
pub async fn create_material(
    State(state): State<AppState>,
    Json(body): Json<CreateMaterialRequest>,
) -> AppResult<Json<AdminMaterialResponse>> {
    let id = Ulid::new().to_string();
    let now = Utc::now().naive_utc();
    let properties_json = body
        .properties
        .as_ref()
        .map(|p| serde_json::to_string(p).unwrap_or_default());

    let material = persistence::materials::create(
        &state.pool,
        &id,
        &body.service_type_id,
        &body.name,
        body.description.as_deref(),
        body.price_per_cm3,
        body.color.as_deref(),
        properties_json.as_deref(),
    )
    .await?;

    let history_id = Ulid::new().to_string();
    persistence::admin::create_pricing_history(
        &state.pool,
        &history_id,
        &id,
        None,
        body.price_per_cm3,
        "admin",
        now,
    )
    .await?;

    tracing::info!("Admin created material: {} ({})", body.name, id);
    Ok(Json(material.into()))
}

#[derive(Deserialize)]
pub struct UpdateMaterialRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub price_per_cm3: Option<f64>,
    pub color: Option<String>,
    pub properties: Option<serde_json::Value>,
    pub active: Option<bool>,
}

/// Update material properties
pub async fn update_material(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<UpdateMaterialRequest>,
) -> AppResult<Json<AdminMaterialResponse>> {
    let current = persistence::materials::find_by_id(&state.pool, &id)
        .await?
        .ok_or_else(|| AppError::MaterialNotFound(id.clone()))?;
    let now = Utc::now().naive_utc();

    if let Some(new_price) = body.price_per_cm3
        && (new_price - current.price_per_cm3).abs() > f64::EPSILON
    {
        let history_id = Ulid::new().to_string();
        persistence::admin::create_pricing_history(
            &state.pool,
            &history_id,
            &id,
            Some(current.price_per_cm3),
            new_price,
            "admin",
            now,
        )
        .await?;
        tracing::info!(
            "Price changed for material {}: {}€ -> {}€",
            id,
            current.price_per_cm3,
            new_price
        );
    }

    let name = body.name.as_deref().or(Some(current.name.as_str()));
    let description = body
        .description
        .as_deref()
        .or(current.description.as_deref());
    let price_per_cm3 = body.price_per_cm3.or(Some(current.price_per_cm3));
    let color = body.color.as_deref().or(current.color.as_deref());
    // Convert current properties from String to Value if needed
    let current_properties_value = current
        .properties
        .and_then(|s| serde_json::from_str(&s).ok());
    let properties = body.properties.or(current_properties_value);
    let active = body.active.or(Some(current.active));
    let properties_json = properties
        .as_ref()
        .map(|p| serde_json::to_string(p).unwrap_or_default());

    let updated = persistence::materials::update(
        &state.pool,
        &id,
        name,
        description,
        price_per_cm3,
        color,
        properties_json.as_deref(),
        active,
    )
    .await?;

    tracing::info!("Admin updated material: {}", id);
    Ok(Json(updated.into()))
}

#[derive(Serialize)]
pub struct PricingHistoryEntry {
    pub id: String,
    pub material_id: String,
    pub material_name: String,
    pub old_price: Option<f64>,
    pub new_price: f64,
    pub changed_by: Option<String>,
    pub changed_at: String,
}

/// Get pricing change history
pub async fn get_pricing_history(
    State(state): State<AppState>,
) -> AppResult<Json<Vec<PricingHistoryEntry>>> {
    let entries = persistence::admin::get_pricing_history(&state.pool).await?;

    let history: Vec<PricingHistoryEntry> = entries
        .into_iter()
        .map(
            |(id, material_id, old_price, new_price, changed_by, changed_at, material_name)| {
                PricingHistoryEntry {
                    id,
                    material_id,
                    material_name,
                    old_price,
                    new_price,
                    changed_by,
                    changed_at,
                }
            },
        )
        .collect();

    tracing::info!("Admin fetched {} pricing history entries", history.len());
    Ok(Json(history))
}

/// Cleanup expired sessions and their associated files
pub async fn cleanup_expired_sessions(
    State(state): State<AppState>,
) -> AppResult<Json<CleanupResult>> {
    let session_service = SessionService::new(state.pool.clone(), &state.config.upload_dir);

    let result = session_service
        .cleanup_expired()
        .await
        .map_err(|e| AppError::Internal(format!("Cleanup failed: {}", e)))?;

    tracing::info!(
        "Admin triggered cleanup: {} sessions, {} directories, {} errors",
        result.sessions_deleted,
        result.directories_deleted,
        result.errors.len()
    );

    Ok(Json(result))
}

use axum::{
    extract::{Path, State},
    Json,
};
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use chrono::Utc;

use crate::api::middleware::{AppResult, AppError};
use crate::api::routes::AppState;
use crate::models::material::Material;

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
    pub created_at: String,
    pub updated_at: Option<String>,
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
            updated_at: if m.updated_at.is_empty() { None } else { Some(m.updated_at) },
        }
    }
}

/// List all materials (admin view - includes inactive)
pub async fn list_materials(
    State(state): State<AppState>,
) -> AppResult<Json<Vec<AdminMaterialResponse>>> {
    let materials: Vec<Material> = sqlx::query_as(
        r#"
        SELECT id, service_type_id, name, description, price_per_cm3,
               color, properties, active, created_at, updated_at
        FROM materials
        ORDER BY name
        "#,
    )
    .fetch_all(&state.pool)
    .await?;

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
    let now = Utc::now().to_rfc3339();
    let properties_json = body.properties.as_ref().map(|p| serde_json::to_string(p).unwrap_or_default());

    sqlx::query(
        r#"INSERT INTO materials (id, service_type_id, name, description, price_per_cm3, color, properties, active, created_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, 1, ?)"#,
    )
    .bind(&id).bind(&body.service_type_id).bind(&body.name).bind(&body.description)
    .bind(body.price_per_cm3).bind(&body.color).bind(&properties_json).bind(&now)
    .execute(&state.pool).await?;

    let history_id = Ulid::new().to_string();
    sqlx::query(r#"INSERT INTO pricing_history (id, material_id, old_price, new_price, changed_by, changed_at) VALUES (?, ?, NULL, ?, ?, ?)"#)
        .bind(&history_id).bind(&id).bind(body.price_per_cm3).bind("admin").bind(&now)
        .execute(&state.pool).await?;

    let material: Material = sqlx::query_as(
        r#"SELECT id, service_type_id, name, description, price_per_cm3, color, properties, active, created_at, updated_at FROM materials WHERE id = ?"#,
    ).bind(&id).fetch_one(&state.pool).await?;

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
    let current: Option<Material> = sqlx::query_as(
        r#"SELECT id, service_type_id, name, description, price_per_cm3, color, properties, active, created_at, updated_at FROM materials WHERE id = ?"#,
    ).bind(&id).fetch_optional(&state.pool).await?;

    let current = current.ok_or_else(|| AppError::MaterialNotFound(id.clone()))?;
    let now = Utc::now().to_rfc3339();

    if let Some(new_price) = body.price_per_cm3 {
        if (new_price - current.price_per_cm3).abs() > f64::EPSILON {
            let history_id = Ulid::new().to_string();
            sqlx::query(r#"INSERT INTO pricing_history (id, material_id, old_price, new_price, changed_by, changed_at) VALUES (?, ?, ?, ?, ?, ?)"#)
                .bind(&history_id).bind(&id).bind(current.price_per_cm3).bind(new_price).bind("admin").bind(&now)
                .execute(&state.pool).await?;
            tracing::info!("Price changed for material {}: {}€ -> {}€", id, current.price_per_cm3, new_price);
        }
    }

    let name = body.name.unwrap_or(current.name);
    let description = body.description.or(current.description);
    let price_per_cm3 = body.price_per_cm3.unwrap_or(current.price_per_cm3);
    let color = body.color.or(current.color);
    // Convert current properties from String to Value if needed
    let current_properties_value = current.properties.and_then(|s| serde_json::from_str(&s).ok());
    let properties = body.properties.or(current_properties_value);
    let active = body.active.unwrap_or(current.active);
    let properties_json = properties.as_ref().map(|p| serde_json::to_string(p).unwrap_or_default());

    sqlx::query(r#"UPDATE materials SET name = ?, description = ?, price_per_cm3 = ?, color = ?, properties = ?, active = ?, updated_at = ? WHERE id = ?"#)
        .bind(&name).bind(&description).bind(price_per_cm3).bind(&color).bind(&properties_json).bind(active).bind(&now).bind(&id)
        .execute(&state.pool).await?;

    let updated: Material = sqlx::query_as(
        r#"SELECT id, service_type_id, name, description, price_per_cm3, color, properties, active, created_at, updated_at FROM materials WHERE id = ?"#,
    ).bind(&id).fetch_one(&state.pool).await?;

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
    let entries: Vec<(String, String, Option<f64>, f64, Option<String>, String, String)> = sqlx::query_as(
        r#"SELECT ph.id, ph.material_id, ph.old_price, ph.new_price, ph.changed_by, ph.changed_at, m.name
        FROM pricing_history ph JOIN materials m ON ph.material_id = m.id ORDER BY ph.changed_at DESC LIMIT 100"#,
    ).fetch_all(&state.pool).await?;

    let history: Vec<PricingHistoryEntry> = entries.into_iter().map(|(id, material_id, old_price, new_price, changed_by, changed_at, material_name)| {
        PricingHistoryEntry { id, material_id, material_name, old_price, new_price, changed_by, changed_at }
    }).collect();

    tracing::info!("Admin fetched {} pricing history entries", history.len());
    Ok(Json(history))
}

/// Cleanup expired sessions and their associated files
pub async fn cleanup_expired_sessions(
    State(state): State<AppState>,
) -> AppResult<Json<crate::services::CleanupResult>> {
    let session_service = crate::services::SessionService::new(
        state.pool.clone(),
        &state.config.upload_dir,
    );

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

use axum::{
    Json,
    extract::{Query, State},
};
use serde::{Deserialize, Serialize};

use crate::api::middleware::AppResult;
use crate::api::routes::AppState;
use crate::persistence;

#[derive(Deserialize)]
pub struct MaterialsQuery {
    pub service_type: Option<String>,
}

#[derive(Serialize)]
pub struct MaterialResponse {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub price_per_cm3: f64,
    pub color: Option<String>,
    pub properties: Option<serde_json::Value>,
}

/// List available materials
pub async fn list_materials(
    State(state): State<AppState>,
    Query(query): Query<MaterialsQuery>,
) -> AppResult<Json<Vec<MaterialResponse>>> {
    let materials = if let Some(service_type) = query.service_type {
        persistence::materials::list_by_service_type(&state.pool, &service_type).await?
    } else {
        persistence::materials::list_all_active(&state.pool).await?
    };

    let response: Vec<MaterialResponse> = materials
        .into_iter()
        .map(|m| MaterialResponse {
            id: m.id,
            name: m.name,
            description: m.description,
            price_per_cm3: m.price_per_cm3,
            color: m.color,
            properties: m.properties.and_then(|p| serde_json::from_str(&p).ok()),
        })
        .collect();

    tracing::info!("Listed {} materials", response.len());

    Ok(Json(response))
}

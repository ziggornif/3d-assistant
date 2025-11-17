use axum::{
    Json,
    extract::{Query, State},
};
use serde::{Deserialize, Serialize};

use crate::api::middleware::AppResult;
use crate::api::routes::AppState;

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
    let materials: Vec<crate::models::material::Material> =
        if let Some(service_type) = query.service_type {
            sqlx::query_as(
                r#"
            SELECT m.id, m.service_type_id, m.name, m.description, m.price_per_cm3,
                   m.color, m.properties, m.active, m.created_at, m.updated_at
            FROM materials m
            JOIN service_types st ON m.service_type_id = st.id
            WHERE st.name = $1 AND m.active = true
            ORDER BY m.name
            "#,
            )
            .bind(&service_type)
            .fetch_all(&state.pool)
            .await?
        } else {
            sqlx::query_as(
                r#"
            SELECT id, service_type_id, name, description, price_per_cm3,
                   color, properties, active, created_at, updated_at
            FROM materials
            WHERE active = true
            ORDER BY name
            "#,
            )
            .fetch_all(&state.pool)
            .await?
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

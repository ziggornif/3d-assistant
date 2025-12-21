//! List materials tool implementation

use crate::{mcp::types::MaterialInfo, persistence};
use sqlx::PgPool;

/// List all available printing materials
pub async fn list_materials(pool: PgPool) -> Result<Vec<MaterialInfo>, String> {
    let materials = persistence::materials::list_all_active(&pool)
        .await
        .map_err(|e| format!("Failed to list materials: {}", e))?;

    let result: Vec<MaterialInfo> = materials
        .into_iter()
        .map(|m| MaterialInfo {
            id: m.id,
            name: m.name,
            description: m.description,
            price_per_cm3: m.price_per_cm3,
            color: m.color,
            active: m.active,
        })
        .collect();

    tracing::info!("MCP: Listed {} materials", result.len());

    Ok(result)
}

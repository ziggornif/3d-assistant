//! Generate quote tool implementation

use crate::{
    business::pricing,
    mcp::types::{GenerateQuoteInput, QuoteItem, QuoteResult},
    persistence::{materials, models, quotes},
};
use chrono::Utc;
use sqlx::PgPool;
use ulid::Ulid;

/// Generate a final quote for configured models
pub async fn generate_quote(
    pool: PgPool,
    input: GenerateQuoteInput,
) -> Result<QuoteResult, String> {
    // Get all models with their materials
    let models = models::find_by_session(&pool, &input.session_id)
        .await
        .map_err(|e| format!("Failed to find models: {}", e))?;

    if models.is_empty() {
        return Err("No models in session".to_string());
    }

    // Build quote items
    let mut items = Vec::new();
    for model in &models {
        let material_id = model.material_id.as_ref().ok_or_else(|| {
            format!("Model {} has no material assigned", model.id)
        })?;

        let material = materials::find_by_id(&pool, material_id)
            .await
            .map_err(|e| format!("Failed to find material: {}", e))?;

        if let Some(material) = material {
            let base_volume = model.volume_cm3.unwrap_or(0.0);

            // Calculate support volume based on support analysis
            let support_percentage = model
                .get_support_analysis()
                .map(|s| s.estimated_support_material_percentage as f64)
                .unwrap_or(0.0);

            let support_volume = base_volume * (support_percentage / 100.0);
            let total_volume = base_volume + support_volume;

            let material_cost =
                pricing::calculate_model_price(total_volume, material.price_per_cm3);

            items.push(pricing::QuoteItem {
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
    let breakdown = pricing::generate_quote_breakdown(items.clone());

    // Create quote record
    let quote_id = Ulid::new().to_string();
    let now = Utc::now().naive_utc();
    let breakdown_json = serde_json::to_string(&breakdown)
        .map_err(|e| format!("JSON serialization error: {}", e))?;

    quotes::create(
        &pool,
        &quote_id,
        &input.session_id,
        breakdown.total,
        &breakdown_json,
        "generated",
        now,
    )
    .await
    .map_err(|e| format!("Failed to create quote: {}", e))?;

    let result_items: Vec<QuoteItem> = items
        .iter()
        .map(|item| QuoteItem {
            filename: item.model_name.clone(),
            material: item.material_name.clone(),
            quantity: 1, // Default quantity for now
            volume_cm3: item.volume_cm3,
            unit_price: item.material_cost,
            total_price: item.material_cost,
        })
        .collect();

    tracing::info!(
        "MCP: Generated quote {} for session {}: total={}€",
        quote_id,
        input.session_id,
        breakdown.total
    );

    Ok(QuoteResult {
        quote_id,
        items: result_items,
        subtotal: breakdown.subtotal,
        fees: breakdown.base_fee,
        total: breakdown.total,
        created_at: now.to_string(),
    })
}

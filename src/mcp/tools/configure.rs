//! Configure model tool implementation

use crate::{
    mcp::types::{ConfigureModelInput, ConfigureModelResult},
    persistence::{materials, models},
};
use sqlx::PgPool;

/// Configure a model with material selection and quantity
pub async fn configure_model(
    pool: PgPool,
    input: ConfigureModelInput,
) -> Result<ConfigureModelResult, String> {
    // Fetch the model
    let model = models::find_by_id_and_session(&pool, &input.model_id, &input.session_id)
        .await
        .map_err(|e| format!("Failed to find model: {}", e))?;

    let model = model.ok_or_else(|| format!("Model {} not found in session", input.model_id))?;

    // Fetch the material
    let material = materials::find_by_id(&pool, &input.material_id)
        .await
        .map_err(|e| format!("Failed to find material: {}", e))?;

    let material = material.ok_or_else(|| format!("Material {} not found", input.material_id))?;

    // Validate material is active
    if !material.active {
        return Err(format!("Material {} is not active", input.material_id));
    }

    // Update model with material_id
    models::update_material(&pool, &input.model_id, &input.material_id)
        .await
        .map_err(|e| format!("Failed to update model material: {}", e))?;

    // Calculate estimated price
    let volume = model.volume_cm3.unwrap_or(0.0);
    let unit_price = material.calculate_price(volume);
    let unit_price_f64 = unit_price.to_string().parse::<f64>().unwrap_or(0.0);
    let estimated_price = unit_price_f64 * input.quantity as f64;

    tracing::info!(
        "MCP: Configured model {} with material {} ({}), quantity: {}, estimated price: {}€",
        input.model_id,
        material.name,
        input.material_id,
        input.quantity,
        estimated_price
    );

    Ok(ConfigureModelResult {
        model_id: input.model_id,
        material_id: input.material_id,
        quantity: input.quantity,
        estimated_price,
    })
}

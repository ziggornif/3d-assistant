use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Represents a printing material with pricing information
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Material {
    pub id: String,
    pub service_type_id: String,
    pub name: String,
    pub description: Option<String>,
    pub price_per_cm3: f64,
    pub color: Option<String>,
    pub properties: Option<String>, // JSON string
    pub active: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// Material properties stored as JSON
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialProperties {
    pub density: Option<f64>,           // g/cm³
    pub print_temperature: Option<i32>, // °C
    pub bed_temperature: Option<i32>,   // °C
    pub strength: Option<String>,       // e.g., "high", "medium", "low"
    pub flexibility: Option<String>,
}

impl Material {
    /// Create a new material
    #[allow(dead_code)]
    pub fn new(
        id: String,
        service_type_id: String,
        name: String,
        description: Option<String>,
        price_per_cm3: f64,
        color: Option<String>,
    ) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id,
            service_type_id,
            name,
            description,
            price_per_cm3,
            color,
            properties: None,
            active: true,
            created_at: now.clone(),
            updated_at: now,
        }
    }

    /// Calculate price for a given volume
    pub fn calculate_price(&self, volume_cm3: f64) -> Decimal {
        let price = self.price_per_cm3 * volume_cm3;
        Decimal::from_f64_retain(price)
            .unwrap_or_default()
            .round_dp(2)
    }
}

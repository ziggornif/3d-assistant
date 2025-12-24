use chrono::NaiveDateTime;
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
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone)]
pub struct CreateMaterial<'a> {
    pub id: &'a str,
    pub service_type_id: &'a str,
    pub name: &'a str,
    pub description: Option<&'a str>,
    pub price_per_cm3: f64,
    pub color: Option<&'a str>,
    pub properties: Option<&'a str>,
}

#[derive(Debug, Clone)]
pub struct UpdateMaterial<'a> {
    pub id: &'a str,
    pub name: Option<&'a str>,
    pub description: Option<&'a str>,
    pub price_per_cm3: Option<f64>,
    pub color: Option<&'a str>,
    pub properties: Option<&'a str>,
    pub active: Option<bool>,
}

impl Material {
    /// Calculate price for a given volume
    pub fn calculate_price(&self, volume_cm3: f64) -> Decimal {
        let price = self.price_per_cm3 * volume_cm3;
        Decimal::from_f64_retain(price)
            .unwrap_or_default()
            .round_dp(2)
    }
}

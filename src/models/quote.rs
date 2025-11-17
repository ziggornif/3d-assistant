use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use ulid::Ulid;

/// Represents a user's quote session
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct QuoteSession {
    pub id: String,
    pub created_at: String,
    pub expires_at: String,
    pub status: String,
}

impl QuoteSession {
    /// Create a new session with default 24h expiration
    pub fn new() -> Self {
        let now = Utc::now();
        let expires = now + Duration::hours(24);

        Self {
            id: Ulid::new().to_string(),
            created_at: now.to_rfc3339(),
            expires_at: expires.to_rfc3339(),
            status: "active".to_string(),
        }
    }

    /// Check if session is expired
    pub fn is_expired(&self) -> bool {
        if let Ok(expires) = DateTime::parse_from_rfc3339(&self.expires_at) {
            expires < Utc::now()
        } else {
            true
        }
    }
}

impl Default for QuoteSession {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents an uploaded 3D model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UploadedModel {
    pub id: String,
    pub session_id: String,
    pub filename: String,
    pub file_format: String,
    pub file_size_bytes: i64,
    pub volume_cm3: Option<f64>,
    pub dimensions_mm: Option<String>, // JSON: {x, y, z}
    pub triangle_count: Option<i32>,
    pub material_id: Option<String>,
    pub file_path: String,
    pub created_at: String,
    pub support_analysis: Option<String>, // JSON: {needs_support, overhang_percentage, estimated_support_material_percentage}
}

/// Dimensions of a 3D model in millimeters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dimensions {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl UploadedModel {
    /// Create a new uploaded model
    pub fn new(
        session_id: String,
        filename: String,
        file_format: String,
        file_size_bytes: i64,
        file_path: String,
    ) -> Self {
        Self {
            id: Ulid::new().to_string(),
            session_id,
            filename,
            file_format,
            file_size_bytes,
            volume_cm3: None,
            dimensions_mm: None,
            triangle_count: None,
            material_id: None,
            file_path,
            created_at: Utc::now().to_rfc3339(),
            support_analysis: None,
        }
    }

    /// Set model dimensions
    pub fn set_dimensions(&mut self, dimensions: Dimensions) {
        self.dimensions_mm = Some(serde_json::to_string(&dimensions).unwrap_or_default());
    }

    /// Get model dimensions
    pub fn get_dimensions(&self) -> Option<Dimensions> {
        self.dimensions_mm
            .as_ref()
            .and_then(|s| serde_json::from_str(s).ok())
    }

    /// Set support analysis data
    pub fn set_support_analysis(
        &mut self,
        analysis: crate::services::file_processor::SupportAnalysis,
    ) {
        self.support_analysis = Some(serde_json::to_string(&analysis).unwrap_or_default());
    }

    /// Get support analysis data
    pub fn get_support_analysis(&self) -> Option<crate::services::file_processor::SupportAnalysis> {
        self.support_analysis
            .as_ref()
            .and_then(|s| serde_json::from_str(s).ok())
    }
}

/// Represents a price quote
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Quote {
    pub id: String,
    pub session_id: String,
    pub total_price: f64,
    pub breakdown: String, // JSON
    pub created_at: String,
}

/// Quote breakdown item
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteItem {
    pub model_id: String,
    pub model_name: String,
    pub material_name: String,
    pub volume_cm3: f64,
    pub material_cost: f64,
    pub subtotal: f64,
}

/// Full quote breakdown
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteBreakdown {
    pub items: Vec<QuoteItem>,
    pub subtotal: f64,
    pub base_fee: f64,
    pub total: f64,
}

impl Quote {
    /// Create a new quote
    #[allow(dead_code)]
    pub fn new(session_id: String, breakdown: QuoteBreakdown) -> Self {
        Self {
            id: Ulid::new().to_string(),
            session_id,
            total_price: breakdown.total,
            breakdown: serde_json::to_string(&breakdown).unwrap_or_default(),
            created_at: Utc::now().to_rfc3339(),
        }
    }

    /// Get the quote breakdown
    #[allow(dead_code)]
    pub fn get_breakdown(&self) -> Option<QuoteBreakdown> {
        serde_json::from_str(&self.breakdown).ok()
    }
}

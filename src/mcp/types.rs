//! Type definitions for MCP tools

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Input parameters for upload_model tool
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UploadModelInput {
    /// Session ID to associate the model with
    pub session_id: String,
    /// Base64-encoded file data (STL or 3MF)
    pub file_data: String,
    /// Original filename
    pub filename: String,
}

/// Result of uploading a model
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UploadModelResult {
    /// Unique identifier for the uploaded model
    pub model_id: String,
    /// Filename as stored
    pub filename: String,
    /// File format detected (stl or 3mf)
    pub file_format: String,
    /// Volume in cubic centimeters
    pub volume_cm3: Option<f64>,
    /// Dimensions in millimeters
    pub dimensions_mm: Option<Dimensions>,
    /// Number of triangles in the mesh
    pub triangle_count: Option<i32>,
}

/// 3D model dimensions
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Dimensions {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// Input parameters for configure_model tool
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ConfigureModelInput {
    /// Session ID
    pub session_id: String,
    /// Model ID to configure
    pub model_id: String,
    /// Material ID to use for printing
    pub material_id: String,
    /// Quantity to print (default: 1)
    #[serde(default = "default_quantity")]
    pub quantity: i32,
}

fn default_quantity() -> i32 {
    1
}

/// Result of configuring a model
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ConfigureModelResult {
    /// Model ID that was configured
    pub model_id: String,
    /// Material ID assigned
    pub material_id: String,
    /// Quantity
    pub quantity: i32,
    /// Estimated price for this model
    pub estimated_price: f64,
}

/// Material information
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MaterialInfo {
    /// Unique material identifier
    pub id: String,
    /// Material name
    pub name: String,
    /// Material description
    pub description: Option<String>,
    /// Price per cubic centimeter
    pub price_per_cm3: f64,
    /// Material color
    pub color: Option<String>,
    /// Whether this material is currently available
    pub active: bool,
}

/// Input for generate_quote tool
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GenerateQuoteInput {
    /// Session ID containing the configured models
    pub session_id: String,
}

/// Quote breakdown item
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct QuoteItem {
    /// Model filename
    pub filename: String,
    /// Material name
    pub material: String,
    /// Quantity
    pub quantity: i32,
    /// Volume in cm³
    pub volume_cm3: f64,
    /// Unit price
    pub unit_price: f64,
    /// Total price for this item (unit_price × quantity)
    pub total_price: f64,
}

/// Generated quote result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct QuoteResult {
    /// Unique quote identifier
    pub quote_id: String,
    /// List of items in the quote
    pub items: Vec<QuoteItem>,
    /// Subtotal before fees
    pub subtotal: f64,
    /// Additional fees
    pub fees: f64,
    /// Total price
    pub total: f64,
    /// Quote creation timestamp
    pub created_at: String,
}

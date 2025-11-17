use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Represents a type of manufacturing service (3D printing, laser cutting, etc.)
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ServiceType {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub active: bool,
    pub created_at: String,
}

impl ServiceType {
    /// Create a new service type
    #[allow(dead_code)]
    pub fn new(id: String, name: String, description: Option<String>) -> Self {
        Self {
            id,
            name,
            description,
            active: true,
            created_at: chrono::Utc::now().to_rfc3339(),
        }
    }
}

/// Service type IDs
#[allow(dead_code)]
pub mod types {
    pub const PRINTING_3D: &str = "3d_printing";
    pub const LASER_CUTTING: &str = "laser_cutting";
    pub const ENGRAVING: &str = "engraving";
}

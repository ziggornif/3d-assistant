use chrono::{Duration, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use ulid::Ulid;

/// Session type constants
pub const SESSION_TYPE_ANONYMOUS: &str = "anonymous";
pub const SESSION_TYPE_AUTHENTICATED: &str = "authenticated";

/// Represents a user's quote session
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct QuoteSession {
    pub id: String,
    pub created_at: NaiveDateTime,
    pub expires_at: NaiveDateTime,
    pub status: String,
    pub user_id: Option<String>,
    pub session_type: String,
}

impl QuoteSession {
    /// Create a new anonymous session with default 24h expiration
    pub fn new() -> Self {
        let now = Utc::now();
        let expires = now + Duration::hours(24);

        Self {
            id: Ulid::new().to_string(),
            created_at: now.naive_utc(),
            expires_at: expires.naive_utc(),
            status: "active".to_string(),
            user_id: None,
            session_type: SESSION_TYPE_ANONYMOUS.to_string(),
        }
    }

    /// Create a new authenticated session linked to a user (30 days expiration)
    pub fn new_authenticated(user_id: String) -> Self {
        let now = Utc::now();
        let expires = now + Duration::days(30);

        Self {
            id: Ulid::new().to_string(),
            created_at: now.naive_utc(),
            expires_at: expires.naive_utc(),
            status: "active".to_string(),
            user_id: Some(user_id),
            session_type: SESSION_TYPE_AUTHENTICATED.to_string(),
        }
    }

    /// Check if session is expired
    pub fn is_expired(&self) -> bool {
        self.expires_at < Utc::now().naive_utc()
    }

    /// Check if this is an anonymous (demo) session
    pub fn is_anonymous(&self) -> bool {
        self.session_type == SESSION_TYPE_ANONYMOUS
    }

    /// Check if this is an authenticated session
    pub fn is_authenticated(&self) -> bool {
        self.session_type == SESSION_TYPE_AUTHENTICATED
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
    pub triangle_count: Option<i64>,
    pub material_id: Option<String>,
    pub file_path: String,
    pub preview_url: String,
    pub created_at: NaiveDateTime,
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
        let id = Ulid::new().to_string();
        let preview_url = format!("/uploads/{}/{}.{}", &session_id, &id, &file_format);
        Self {
            id,
            session_id,
            filename,
            file_format,
            file_size_bytes,
            volume_cm3: None,
            dimensions_mm: None,
            triangle_count: None,
            material_id: None,
            file_path,
            preview_url,
            created_at: Utc::now().naive_utc(),
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
        analysis: crate::business::file_processor::SupportAnalysis,
    ) {
        self.support_analysis = Some(serde_json::to_string(&analysis).unwrap_or_default());
    }

    /// Get support analysis data
    pub fn get_support_analysis(&self) -> Option<crate::business::file_processor::SupportAnalysis> {
        self.support_analysis
            .as_ref()
            .and_then(|s| serde_json::from_str(s).ok())
    }
}

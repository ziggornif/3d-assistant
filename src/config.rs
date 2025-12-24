use anyhow::{anyhow, Result};

/// Sanitize sensitive strings for logging
///
/// Replaces sensitive data with `[REDACTED]` to prevent leaking secrets in logs/traces
pub fn sanitize_secret(value: &str) -> String {
    if value.len() > 8 {
        format!("[REDACTED:{}...]", &value[..4])
    } else {
        "[REDACTED]".to_string()
    }
}

/// Application configuration
#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub host: String,
    pub port: u16,
    pub max_file_size_bytes: i64,
    pub upload_dir: String,
    pub static_dir: String,
    pub template_dir: String,
    pub admin_token: String,
    #[allow(dead_code)] // Used by is_production() and future session cleanup
    pub session_expiry_hours: i64,
    #[allow(dead_code)] // Used by is_production()
    pub environment: String,
    // OpenTelemetry configuration
    pub otel_exporter_otlp_endpoint: String,
    pub otel_service_name: String,
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();

        let database_url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://localhost/quotes".to_string());

        let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());

        let port = std::env::var("PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse()
            .unwrap_or(3000);

        let max_file_size_mb: i64 = std::env::var("MAX_FILE_SIZE_MB")
            .unwrap_or_else(|_| "50".to_string())
            .parse()
            .unwrap_or(50);

        let upload_dir = std::env::var("UPLOAD_DIR").unwrap_or_else(|_| "./uploads".to_string());

        let static_dir = std::env::var("STATIC_DIR").unwrap_or_else(|_| "./static".to_string());

        let template_dir =
            std::env::var("TEMPLATE_DIR").unwrap_or_else(|_| "./templates".to_string());

        let admin_token =
            std::env::var("ADMIN_TOKEN").unwrap_or_else(|_| "admin-secret-token-2025".to_string());

        let session_expiry_hours: i64 = std::env::var("SESSION_EXPIRY_HOURS")
            .unwrap_or_else(|_| "24".to_string())
            .parse()
            .unwrap_or(24);

        let environment =
            std::env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string());

        let otel_exporter_otlp_endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
            .unwrap_or_else(|_| "http://localhost:4317".to_string());

        let otel_service_name =
            std::env::var("OTEL_SERVICE_NAME").unwrap_or_else(|_| "quote-service".to_string());

        Ok(Self {
            database_url,
            host,
            port,
            max_file_size_bytes: max_file_size_mb * 1_000_000,
            upload_dir,
            static_dir,
            template_dir,
            admin_token,
            session_expiry_hours,
            environment,
            otel_exporter_otlp_endpoint,
            otel_service_name,
        })
    }

    /// Check if running in production
    pub fn is_production(&self) -> bool {
        self.environment == "production"
    }

    /// Validate required configuration for production deployment
    ///
    /// Checks that critical environment variables are set with non-default values
    /// in production mode. Returns error with clear message if validation fails.
    pub fn validate(&self) -> Result<()> {
        if self.is_production() {
            // Validate DATABASE_URL is set (not default)
            if self.database_url == "postgres://localhost/quotes" {
                return Err(anyhow!(
                    "DATABASE_URL must be set in production. \
                     Please set DATABASE_URL to a valid PostgreSQL connection string."
                ));
            }

            // Validate ADMIN_TOKEN is not default
            if self.admin_token == "admin-secret-token-2025" {
                return Err(anyhow!(
                    "ADMIN_TOKEN must be changed from default value in production. \
                     Please set a secure ADMIN_TOKEN."
                ));
            }

            // Check MCP_TOKEN if needed (optional validation for future MCP auth)
            // This will be enhanced in Phase 3 (US1)
        }

        Ok(())
    }
}

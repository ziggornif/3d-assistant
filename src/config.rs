use anyhow::Result;

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
    #[allow(dead_code)]
    pub session_expiry_hours: i64,
    #[allow(dead_code)]
    pub environment: String,
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
        })
    }

    /// Check if running in production
    #[allow(dead_code)]
    pub fn is_production(&self) -> bool {
        self.environment == "production"
    }
}

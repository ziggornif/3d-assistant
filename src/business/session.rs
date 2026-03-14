use crate::api::middleware::AppError;
use crate::db::DbPool;
use crate::models::QuoteSession;
use crate::persistence;
use anyhow::Result;
use chrono::Utc;
use std::path::{Path, PathBuf};

/// Result of session cleanup operation
#[derive(Debug, Clone, serde::Serialize)]
pub struct CleanupResult {
    pub sessions_deleted: u64,
    pub directories_deleted: u64,
    pub errors: Vec<String>,
}

/// Service for managing quote sessions
pub struct SessionService {
    pool: DbPool,
    upload_dir: PathBuf,
}

impl SessionService {
    pub fn new(pool: DbPool, upload_dir: impl AsRef<Path>) -> Self {
        Self {
            pool,
            upload_dir: upload_dir.as_ref().to_path_buf(),
        }
    }

    /// Create a new anonymous (demo) session
    pub async fn create_session(&self) -> Result<QuoteSession, AppError> {
        let session = QuoteSession::new();

        persistence::sessions::create(
            &self.pool,
            &session.id,
            session.created_at,
            session.expires_at,
            &session.status,
        )
        .await?;

        Ok(session)
    }

    /// Create a new authenticated session linked to a user
    pub async fn create_authenticated_session(
        &self,
        user_id: &str,
    ) -> Result<QuoteSession, AppError> {
        let session = QuoteSession::new_authenticated(user_id.to_string());

        persistence::sessions::create_authenticated(
            &self.pool,
            &session.id,
            user_id,
            session.created_at,
            session.expires_at,
            &session.status,
        )
        .await?;

        Ok(session)
    }

    /// Get session by ID
    pub async fn get_session(&self, session_id: &str) -> Result<QuoteSession, AppError> {
        let session = persistence::sessions::find_by_id(&self.pool, session_id)
            .await?
            .ok_or_else(|| AppError::SessionNotFound(session_id.to_string()))?;

        if session.is_expired() {
            return Err(AppError::SessionExpired(session_id.to_string()));
        }

        Ok(session)
    }

    /// Clean up expired sessions and their associated upload files
    pub async fn cleanup_expired(&self) -> Result<CleanupResult> {
        let mut result = CleanupResult {
            sessions_deleted: 0,
            directories_deleted: 0,
            errors: Vec::new(),
        };

        let now = Utc::now().naive_utc();

        // Get list of expired session IDs
        let expired_sessions = persistence::sessions::find_expired_ids(&self.pool, now).await?;

        // Delete upload directories for each expired session
        for session_id in &expired_sessions {
            let session_upload_dir = self.upload_dir.join(session_id);
            if session_upload_dir.exists() {
                match std::fs::remove_dir_all(&session_upload_dir) {
                    Ok(_) => {
                        tracing::info!("Deleted upload directory for session: {}", session_id);
                        result.directories_deleted += 1;
                    }
                    Err(e) => {
                        let error_msg = format!(
                            "Failed to delete directory for session {}: {}",
                            session_id, e
                        );
                        tracing::error!("{}", error_msg);
                        result.errors.push(error_msg);
                    }
                }
            }
        }

        // Delete data in correct order (foreign key constraints)
        persistence::models::delete_by_expired_sessions(&self.pool, now).await?;
        persistence::quotes::delete_by_expired_sessions(&self.pool, now).await?;
        result.sessions_deleted = persistence::sessions::delete_expired(&self.pool, now).await?;

        tracing::info!(
            "Cleanup completed: {} sessions deleted, {} directories removed, {} errors",
            result.sessions_deleted,
            result.directories_deleted,
            result.errors.len()
        );

        Ok(result)
    }
}

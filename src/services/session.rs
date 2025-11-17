use crate::db::DbPool;
use crate::models::QuoteSession;
use crate::api::middleware::AppError;
use anyhow::Result;
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

    /// Create a new session
    pub async fn create_session(&self) -> Result<QuoteSession, AppError> {
        let session = QuoteSession::new();

        sqlx::query(
            r#"
            INSERT INTO quote_sessions (id, created_at, expires_at, status)
            VALUES (?, ?, ?, ?)
            "#,
        )
        .bind(&session.id)
        .bind(&session.created_at)
        .bind(&session.expires_at)
        .bind(&session.status)
        .execute(&self.pool)
        .await?;

        Ok(session)
    }

    /// Get session by ID
    pub async fn get_session(&self, session_id: &str) -> Result<QuoteSession, AppError> {
        let session: QuoteSession = sqlx::query_as(
            r#"
            SELECT id, created_at, expires_at, status
            FROM quote_sessions
            WHERE id = ?
            "#,
        )
        .bind(session_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AppError::SessionNotFound(session_id.to_string()))?;

        if session.is_expired() {
            return Err(AppError::SessionExpired(session_id.to_string()));
        }

        Ok(session)
    }

    /// Update session status
    #[allow(dead_code)]
    pub async fn update_status(&self, session_id: &str, status: &str) -> Result<(), AppError> {
        sqlx::query(
            r#"
            UPDATE quote_sessions
            SET status = ?
            WHERE id = ?
            "#,
        )
        .bind(status)
        .bind(session_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Clean up expired sessions and their associated upload files
    pub async fn cleanup_expired(&self) -> Result<CleanupResult> {
        let mut result = CleanupResult {
            sessions_deleted: 0,
            directories_deleted: 0,
            errors: Vec::new(),
        };

        // First, get the list of expired session IDs
        let expired_sessions: Vec<(String,)> = sqlx::query_as(
            r#"
            SELECT id FROM quote_sessions
            WHERE expires_at < datetime('now')
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        // Delete upload directories for each expired session
        for (session_id,) in &expired_sessions {
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

        // Delete uploaded_models records for expired sessions
        sqlx::query(
            r#"
            DELETE FROM uploaded_models
            WHERE session_id IN (
                SELECT id FROM quote_sessions
                WHERE expires_at < datetime('now')
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Delete quotes for expired sessions
        sqlx::query(
            r#"
            DELETE FROM quotes
            WHERE session_id IN (
                SELECT id FROM quote_sessions
                WHERE expires_at < datetime('now')
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Finally, delete the expired sessions themselves
        let delete_result = sqlx::query(
            r#"
            DELETE FROM quote_sessions
            WHERE expires_at < datetime('now')
            "#,
        )
        .execute(&self.pool)
        .await?;

        result.sessions_deleted = delete_result.rows_affected();

        tracing::info!(
            "Cleanup completed: {} sessions deleted, {} directories removed, {} errors",
            result.sessions_deleted,
            result.directories_deleted,
            result.errors.len()
        );

        Ok(result)
    }

    /// Get count of expired sessions (without deleting)
    #[allow(dead_code)]
    pub async fn count_expired(&self) -> Result<i64> {
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM quote_sessions
            WHERE expires_at < datetime('now')
            "#,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(count.0)
    }
}

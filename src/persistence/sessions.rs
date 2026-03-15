use crate::models::QuoteSession;
use chrono::NaiveDateTime;
use sqlx::PgPool;

/// Create a new session (with user_id and session_type support)
pub async fn create(
    pool: &PgPool,
    id: &str,
    created_at: NaiveDateTime,
    expires_at: NaiveDateTime,
    status: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r"
        INSERT INTO quote_sessions (id, created_at, expires_at, status, session_type)
        VALUES ($1, $2, $3, $4, 'anonymous')
        ",
    )
    .bind(id)
    .bind(created_at)
    .bind(expires_at)
    .bind(status)
    .execute(pool)
    .await?;

    Ok(())
}

/// Create an authenticated session linked to a user
pub async fn create_authenticated(
    pool: &PgPool,
    id: &str,
    user_id: &str,
    created_at: NaiveDateTime,
    expires_at: NaiveDateTime,
    status: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r"
        INSERT INTO quote_sessions (id, created_at, expires_at, status, user_id, session_type)
        VALUES ($1, $2, $3, $4, $5, 'authenticated')
        ",
    )
    .bind(id)
    .bind(created_at)
    .bind(expires_at)
    .bind(status)
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Get session by ID
pub async fn find_by_id(
    pool: &PgPool,
    session_id: &str,
) -> Result<Option<QuoteSession>, sqlx::Error> {
    sqlx::query_as(
        r"
        SELECT id, created_at, expires_at, status, user_id, session_type
        FROM quote_sessions
        WHERE id = $1
        ",
    )
    .bind(session_id)
    .fetch_optional(pool)
    .await
}

/// Find expired session IDs
pub async fn find_expired_ids(
    pool: &PgPool,
    now: NaiveDateTime,
) -> Result<Vec<String>, sqlx::Error> {
    let results: Vec<(String,)> = sqlx::query_as(
        r"
        SELECT id FROM quote_sessions
        WHERE expires_at < $1
        ",
    )
    .bind(now)
    .fetch_all(pool)
    .await?;

    Ok(results.into_iter().map(|(id,)| id).collect())
}

/// Delete expired sessions
pub async fn delete_expired(pool: &PgPool, now: NaiveDateTime) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        r"
        DELETE FROM quote_sessions
        WHERE expires_at < $1
        ",
    )
    .bind(now)
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

/// Find sessions by user ID (for quote history)
#[allow(dead_code)]
pub async fn find_by_user_id(
    pool: &PgPool,
    user_id: &str,
) -> Result<Vec<QuoteSession>, sqlx::Error> {
    sqlx::query_as(
        r"
        SELECT id, created_at, expires_at, status, user_id, session_type
        FROM quote_sessions
        WHERE user_id = $1
        ORDER BY created_at DESC
        ",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

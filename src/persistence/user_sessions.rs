use crate::models::user::UserSession;
use chrono::NaiveDateTime;
use sqlx::PgPool;

/// Create a new user session
pub async fn create(
    pool: &PgPool,
    token: &str,
    user_id: &str,
    created_at: NaiveDateTime,
    expires_at: NaiveDateTime,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r"
        INSERT INTO user_sessions (token, user_id, created_at, expires_at)
        VALUES ($1, $2, $3, $4)
        ",
    )
    .bind(token)
    .bind(user_id)
    .bind(created_at)
    .bind(expires_at)
    .execute(pool)
    .await?;

    Ok(())
}

/// Find session by token
pub async fn find_by_token(pool: &PgPool, token: &str) -> Result<Option<UserSession>, sqlx::Error> {
    sqlx::query_as(
        r"
        SELECT token, user_id, created_at, expires_at
        FROM user_sessions
        WHERE token = $1
        ",
    )
    .bind(token)
    .fetch_optional(pool)
    .await
}

/// Delete a session by token
pub async fn delete_by_token(pool: &PgPool, token: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        r"
        DELETE FROM user_sessions WHERE token = $1
        ",
    )
    .bind(token)
    .execute(pool)
    .await?;

    Ok(())
}

/// Delete all sessions for a user
pub async fn delete_by_user_id(pool: &PgPool, user_id: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        r"
        DELETE FROM user_sessions WHERE user_id = $1
        ",
    )
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Delete expired user sessions
pub async fn delete_expired(pool: &PgPool, now: NaiveDateTime) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        r"
        DELETE FROM user_sessions WHERE expires_at < $1
        ",
    )
    .bind(now)
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

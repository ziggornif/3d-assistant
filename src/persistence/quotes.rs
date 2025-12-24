use chrono::NaiveDateTime;
use sqlx::PgPool;

/// Create a new quote
pub async fn create(
    pool: &PgPool,
    id: &str,
    session_id: &str,
    total_price: f64,
    breakdown: &str,
    status: &str,
    created_at: NaiveDateTime,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r"
        INSERT INTO quotes (id, session_id, total_price, breakdown, status, created_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        ",
    )
    .bind(id)
    .bind(session_id)
    .bind(total_price)
    .bind(breakdown)
    .bind(status)
    .bind(created_at)
    .execute(pool)
    .await?;

    Ok(())
}

/// Delete quotes for expired sessions
pub async fn delete_by_expired_sessions(
    pool: &PgPool,
    now: NaiveDateTime,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r"
        DELETE FROM quotes
        WHERE session_id IN (
            SELECT id FROM quote_sessions
            WHERE expires_at < $1
        )
        ",
    )
    .bind(now)
    .execute(pool)
    .await?;

    Ok(())
}

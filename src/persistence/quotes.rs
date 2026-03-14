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

/// Soft delete a quote (set deleted_at timestamp)
pub async fn soft_delete(
    pool: &PgPool,
    quote_id: &str,
    user_id: &str,
    now: NaiveDateTime,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        r"
        UPDATE quotes SET status = 'deleted', deleted_at = $1
        WHERE id = $2
          AND status = 'generated'
          AND deleted_at IS NULL
          AND session_id IN (
              SELECT id FROM quote_sessions WHERE user_id = $3
          )
        ",
    )
    .bind(now)
    .bind(quote_id)
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

/// Hard delete a quote (admin only, only already soft-deleted quotes)
pub async fn hard_delete(pool: &PgPool, quote_id: &str) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        r"
        DELETE FROM quotes
        WHERE id = $1 AND status = 'deleted'
        ",
    )
    .bind(quote_id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

/// Find a quote by ID with ownership check (for user endpoints)
pub async fn find_by_id_and_user(
    pool: &PgPool,
    quote_id: &str,
    user_id: &str,
) -> Result<Option<QuoteRow>, sqlx::Error> {
    sqlx::query_as(
        r"
        SELECT q.id, q.session_id, q.total_price, q.status, q.breakdown,
               q.created_at, q.deleted_at
        FROM quotes q
        JOIN quote_sessions qs ON q.session_id = qs.id
        WHERE q.id = $1 AND qs.user_id = $2
        ",
    )
    .bind(quote_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await
}

/// Quote row from database
#[derive(Debug, sqlx::FromRow)]
pub struct QuoteRow {
    pub id: String,
    pub session_id: String,
    pub total_price: f64,
    pub status: String,
    pub breakdown: String,
    pub created_at: NaiveDateTime,
    pub deleted_at: Option<NaiveDateTime>,
}

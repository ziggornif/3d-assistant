use crate::models::user::User;
use chrono::NaiveDateTime;
use sqlx::PgPool;

/// Create a new user
pub async fn create(
    pool: &PgPool,
    id: &str,
    email: &str,
    password_hash: &str,
    display_name: &str,
    status: &str,
    role: &str,
    created_at: NaiveDateTime,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r"
        INSERT INTO users (id, email, password_hash, display_name, status, role, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $7)
        ",
    )
    .bind(id)
    .bind(email)
    .bind(password_hash)
    .bind(display_name)
    .bind(status)
    .bind(role)
    .bind(created_at)
    .execute(pool)
    .await?;

    Ok(())
}

/// Find user by email
pub async fn find_by_email(pool: &PgPool, email: &str) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as(
        r"
        SELECT id, email, password_hash, display_name, status, role, created_at, updated_at
        FROM users
        WHERE email = $1
        ",
    )
    .bind(email)
    .fetch_optional(pool)
    .await
}

/// Find user by ID
pub async fn find_by_id(pool: &PgPool, id: &str) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as(
        r"
        SELECT id, email, password_hash, display_name, status, role, created_at, updated_at
        FROM users
        WHERE id = $1
        ",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

/// Update user status
pub async fn update_status(
    pool: &PgPool,
    user_id: &str,
    new_status: &str,
) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as(
        r"
        UPDATE users SET status = $1, updated_at = CURRENT_TIMESTAMP
        WHERE id = $2
        RETURNING id, email, password_hash, display_name, status, role, created_at, updated_at
        ",
    )
    .bind(new_status)
    .bind(user_id)
    .fetch_optional(pool)
    .await
}

/// List users with optional status filter, paginated
pub async fn list_users(
    pool: &PgPool,
    status_filter: Option<&str>,
    limit: i64,
    offset: i64,
) -> Result<Vec<User>, sqlx::Error> {
    if let Some(status) = status_filter {
        sqlx::query_as(
            r"
            SELECT id, email, password_hash, display_name, status, role, created_at, updated_at
            FROM users
            WHERE status = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            ",
        )
        .bind(status)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
    } else {
        sqlx::query_as(
            r"
            SELECT id, email, password_hash, display_name, status, role, created_at, updated_at
            FROM users
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            ",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
    }
}

/// Count users with optional status filter
pub async fn count_users(
    pool: &PgPool,
    status_filter: Option<&str>,
) -> Result<i64, sqlx::Error> {
    let row: (i64,) = if let Some(status) = status_filter {
        sqlx::query_as(
            r"
            SELECT COUNT(*) FROM users WHERE status = $1
            ",
        )
        .bind(status)
        .fetch_one(pool)
        .await?
    } else {
        sqlx::query_as(
            r"
            SELECT COUNT(*) FROM users
            ",
        )
        .fetch_one(pool)
        .await?
    };

    Ok(row.0)
}

/// Count quotes for a given user (via their sessions)
pub async fn count_user_quotes(pool: &PgPool, user_id: &str) -> Result<i64, sqlx::Error> {
    let row: (i64,) = sqlx::query_as(
        r"
        SELECT COUNT(*) FROM quotes q
        JOIN quote_sessions qs ON q.session_id = qs.id
        WHERE qs.user_id = $1
        ",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok(row.0)
}

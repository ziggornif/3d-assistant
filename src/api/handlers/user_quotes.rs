use axum::{
    Json,
    extract::{Path, Query, State},
};
use axum_extra::extract::cookie::CookieJar;
use serde::{Deserialize, Serialize};

use crate::api::middleware::{AppError, AppResult};
use crate::api::routes::AppState;
use crate::business::auth::AuthService;
use crate::persistence;

#[derive(Deserialize)]
pub struct QuotesQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

#[derive(Serialize)]
pub struct QuoteListItem {
    pub id: String,
    pub session_id: String,
    pub total_price: f64,
    pub status: String,
    pub model_count: i64,
    pub created_at: String,
}

#[derive(Serialize)]
pub struct QuoteListResponse {
    pub quotes: Vec<QuoteListItem>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
}

/// Get quote history for the authenticated user
pub async fn list_my_quotes(
    State(state): State<AppState>,
    jar: CookieJar,
    Query(query): Query<QuotesQuery>,
) -> AppResult<Json<QuoteListResponse>> {
    let token = jar
        .get("user_session")
        .ok_or_else(|| AppError::Unauthorized("Non authentifie".to_string()))?;

    let auth_service = AuthService::new(state.pool.clone());
    let user = auth_service
        .verify_session(token.value())
        .await
        .map_err(|_| AppError::Unauthorized("Session invalide ou expiree".to_string()))?;

    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * per_page;

    // Get quotes for user's sessions
    let quotes = get_user_quotes(&state.pool, &user.id, per_page, offset).await?;
    let total = count_user_quotes(&state.pool, &user.id).await?;

    Ok(Json(QuoteListResponse {
        quotes,
        total,
        page,
        per_page,
    }))
}

#[derive(Serialize)]
pub struct QuoteDetailResponse {
    pub id: String,
    pub session_id: String,
    pub total_price: f64,
    pub status: String,
    pub breakdown: serde_json::Value,
    pub created_at: String,
}

/// Get detail of a specific quote for the authenticated user
pub async fn get_my_quote(
    State(state): State<AppState>,
    jar: CookieJar,
    Path(quote_id): Path<String>,
) -> AppResult<Json<QuoteDetailResponse>> {
    let token = jar
        .get("user_session")
        .ok_or_else(|| AppError::Unauthorized("Non authentifie".to_string()))?;

    let auth_service = AuthService::new(state.pool.clone());
    let user = auth_service
        .verify_session(token.value())
        .await
        .map_err(|_| AppError::Unauthorized("Session invalide ou expiree".to_string()))?;

    // Fetch the quote and verify ownership
    let quote = get_quote_detail(&state.pool, &quote_id, &user.id).await?;

    Ok(Json(quote))
}

/// Internal: fetch paginated quotes for a user
async fn get_user_quotes(
    pool: &sqlx::PgPool,
    user_id: &str,
    limit: i64,
    offset: i64,
) -> Result<Vec<QuoteListItem>, AppError> {
    let rows: Vec<(String, String, f64, String, String)> = sqlx::query_as(
        r"
        SELECT q.id, q.session_id, q.total_price, q.status, q.created_at::text
        FROM quotes q
        JOIN quote_sessions qs ON q.session_id = qs.id
        WHERE qs.user_id = $1
        ORDER BY q.created_at DESC
        LIMIT $2 OFFSET $3
        ",
    )
    .bind(user_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    let mut quotes = Vec::new();
    for (id, session_id, total_price, status, created_at) in rows {
        // Count models in this session
        let (model_count,): (i64,) = sqlx::query_as(
            r"SELECT COUNT(*) FROM uploaded_models WHERE session_id = $1",
        )
        .bind(&session_id)
        .fetch_one(pool)
        .await?;

        quotes.push(QuoteListItem {
            id,
            session_id,
            total_price,
            status,
            model_count,
            created_at,
        });
    }

    Ok(quotes)
}

/// Internal: count total quotes for a user
async fn count_user_quotes(pool: &sqlx::PgPool, user_id: &str) -> Result<i64, AppError> {
    let (count,): (i64,) = sqlx::query_as(
        r"
        SELECT COUNT(*)
        FROM quotes q
        JOIN quote_sessions qs ON q.session_id = qs.id
        WHERE qs.user_id = $1
        ",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok(count)
}

/// Internal: fetch quote detail with ownership check
async fn get_quote_detail(
    pool: &sqlx::PgPool,
    quote_id: &str,
    user_id: &str,
) -> Result<QuoteDetailResponse, AppError> {
    let row: Option<(String, String, f64, String, String, String)> = sqlx::query_as(
        r"
        SELECT q.id, q.session_id, q.total_price, q.status, q.breakdown, q.created_at::text
        FROM quotes q
        JOIN quote_sessions qs ON q.session_id = qs.id
        WHERE q.id = $1 AND qs.user_id = $2
        ",
    )
    .bind(quote_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    let (id, session_id, total_price, status, breakdown_str, created_at) = row
        .ok_or_else(|| AppError::NotFound("Devis non trouvé".to_string()))?;

    let breakdown: serde_json::Value =
        serde_json::from_str(&breakdown_str).unwrap_or(serde_json::json!({}));

    Ok(QuoteDetailResponse {
        id,
        session_id,
        total_price,
        status,
        breakdown,
        created_at,
    })
}

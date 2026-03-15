use axum::{
    Json,
    extract::{Path, Query, State},
    http::{StatusCode, header},
    response::IntoResponse,
};
use axum_extra::extract::cookie::CookieJar;
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use crate::api::middleware::{AppError, AppResult};
use crate::api::routes::AppState;
use crate::business::auth::AuthService;
use crate::persistence;

#[derive(Deserialize)]
pub struct QuotesQuery {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub status: Option<String>,
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

/// Helper: verify user auth from cookie jar
async fn verify_user(state: &AppState, jar: &CookieJar) -> Result<crate::models::User, AppError> {
    let token = jar
        .get("user_session")
        .ok_or_else(|| AppError::Unauthorized("Non authentifie".to_string()))?;

    let auth_service = AuthService::new(state.pool.clone());
    auth_service
        .verify_session(token.value())
        .await
        .map_err(|_| AppError::Unauthorized("Session invalide ou expiree".to_string()))
}

/// Get quote history for the authenticated user (includes drafts)
pub async fn list_my_quotes(
    State(state): State<AppState>,
    jar: CookieJar,
    Query(query): Query<QuotesQuery>,
) -> AppResult<Json<QuoteListResponse>> {
    let user = verify_user(&state, &jar).await?;

    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * per_page;
    let status_filter = query.status.as_deref();

    let mut all_items: Vec<QuoteListItem> = Vec::new();

    // Include drafts if filter allows it
    let include_drafts = matches!(status_filter, None | Some("draft"));
    let include_generated = matches!(status_filter, None | Some("generated"));
    let include_deleted = matches!(status_filter, Some("deleted"));

    // Fetch generated/deleted quotes from DB
    if include_generated || include_deleted {
        let quote_status = if include_deleted {
            "deleted"
        } else {
            "generated"
        };

        let rows: Vec<(String, String, f64, String, String, i64)> = sqlx::query_as(
            r"
            SELECT q.id, q.session_id, q.total_price, q.status, q.created_at::text,
                   (SELECT COUNT(*) FROM uploaded_models um WHERE um.session_id = q.session_id)
            FROM quotes q
            JOIN quote_sessions qs ON q.session_id = qs.id
            WHERE qs.user_id = $1 AND q.status = $2
            ORDER BY q.created_at DESC
            ",
        )
        .bind(&user.id)
        .bind(quote_status)
        .fetch_all(&state.pool)
        .await?;

        for (id, session_id, total_price, status, created_at, model_count) in rows {
            all_items.push(QuoteListItem {
                id,
                session_id,
                total_price,
                status,
                model_count,
                created_at,
            });
        }
    }

    // Fetch drafts: authenticated sessions with models but no quote
    if include_drafts {
        let draft_rows: Vec<(String, String, i64)> = sqlx::query_as(
            r"
            SELECT qs.id, qs.created_at::text,
                   (SELECT COUNT(*) FROM uploaded_models um WHERE um.session_id = qs.id)
            FROM quote_sessions qs
            WHERE qs.user_id = $1
              AND qs.session_type = 'authenticated'
              AND qs.expires_at > NOW()
              AND (SELECT COUNT(*) FROM uploaded_models um WHERE um.session_id = qs.id) > 0
              AND NOT EXISTS (
                  SELECT 1 FROM quotes q WHERE q.session_id = qs.id AND q.status != 'deleted'
              )
            ORDER BY qs.created_at DESC
            ",
        )
        .bind(&user.id)
        .fetch_all(&state.pool)
        .await?;

        for (session_id, created_at, model_count) in draft_rows {
            all_items.push(QuoteListItem {
                id: session_id.clone(),
                session_id,
                total_price: 0.0,
                status: "draft".to_string(),
                model_count,
                created_at,
            });
        }
    }

    // Sort by created_at descending (drafts and quotes mixed)
    all_items.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    let total = all_items.len() as i64;

    // Apply pagination
    let offset_usize = offset as usize;
    let per_page_usize = per_page as usize;
    let paginated: Vec<QuoteListItem> = all_items
        .into_iter()
        .skip(offset_usize)
        .take(per_page_usize)
        .collect();

    Ok(Json(QuoteListResponse {
        quotes: paginated,
        total,
        page,
        per_page,
    }))
}

#[derive(Serialize)]
pub struct QuoteDetailModel {
    pub id: String,
    pub filename: String,
    pub volume_cm3: f64,
    pub material_name: String,
    pub price: f64,
}

#[derive(Serialize)]
pub struct QuoteDetailResponse {
    pub id: String,
    pub session_id: String,
    pub total_price: f64,
    pub status: String,
    pub breakdown: serde_json::Value,
    pub created_at: String,
    pub models: Vec<QuoteDetailModel>,
}

/// Get detail of a specific quote for the authenticated user
pub async fn get_my_quote(
    State(state): State<AppState>,
    jar: CookieJar,
    Path(quote_id): Path<String>,
) -> AppResult<Json<QuoteDetailResponse>> {
    let user = verify_user(&state, &jar).await?;
    let quote = get_quote_detail(&state.pool, &quote_id, &user.id).await?;
    Ok(Json(quote))
}

/// Soft delete a quote
#[derive(Deserialize)]
pub struct UpdateQuoteStatusRequest {
    pub status: String,
}

pub async fn soft_delete_quote(
    State(state): State<AppState>,
    jar: CookieJar,
    Path(quote_id): Path<String>,
    Json(body): Json<UpdateQuoteStatusRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let user = verify_user(&state, &jar).await?;

    if body.status != "deleted" {
        return Err(AppError::Validation(
            "Seule la transition vers 'deleted' est autorisee".to_string(),
        ));
    }

    let now = chrono::Utc::now().naive_utc();
    let deleted = persistence::quotes::soft_delete(&state.pool, &quote_id, &user.id, now).await?;

    if !deleted {
        return Err(AppError::NotFound(
            "Devis non trouve, deja supprime, ou non autorise".to_string(),
        ));
    }

    tracing::info!("User {} soft-deleted quote {}", user.id, quote_id);

    Ok(Json(serde_json::json!({
        "id": quote_id,
        "status": "deleted",
        "deleted_at": now.to_string()
    })))
}

/// Export a quote as CSV
pub async fn export_quote(
    State(state): State<AppState>,
    jar: CookieJar,
    Path(quote_id): Path<String>,
    Query(params): Query<ExportQuery>,
) -> Result<impl IntoResponse, AppError> {
    let user = verify_user(&state, &jar).await?;

    let format = params.format.as_deref().unwrap_or("csv");
    if format != "csv" {
        return Err(AppError::Validation(format!(
            "Format '{}' non supporte. Formats disponibles: csv",
            format
        )));
    }

    // Fetch quote with ownership check
    let quote_row = persistence::quotes::find_by_id_and_user(&state.pool, &quote_id, &user.id)
        .await?
        .ok_or_else(|| AppError::NotFound("Devis non trouve".to_string()))?;

    if quote_row.status != "generated" {
        return Err(AppError::Validation(
            "L'export n'est disponible que pour les devis generes".to_string(),
        ));
    }

    // Fetch models for the session
    #[allow(clippy::type_complexity)]
    let model_rows: Vec<(String, Option<f64>, Option<String>, Option<f64>)> = sqlx::query_as(
        r"
        SELECT um.filename, um.volume_cm3, m.name, m.price_per_cm3
        FROM uploaded_models um
        LEFT JOIN materials m ON um.material_id = m.id
        WHERE um.session_id = $1
        ",
    )
    .bind(&quote_row.session_id)
    .fetch_all(&state.pool)
    .await?;

    // Parse breakdown for totals
    let breakdown: serde_json::Value =
        serde_json::from_str(&quote_row.breakdown).unwrap_or(serde_json::json!({}));

    // Build CSV content
    let date_str = quote_row.created_at.format("%Y-%m-%d").to_string();
    let id_short = if quote_row.id.len() >= 8 {
        &quote_row.id[..8]
    } else {
        &quote_row.id
    };

    // UTF-8 BOM for Excel compatibility
    let mut csv = String::from("\u{FEFF}");
    csv.push_str(&format!("Devis;{}\n", quote_row.id));
    csv.push_str(&format!("Date;{}\n", date_str));
    csv.push('\n');
    csv.push_str("Modele;Materiau;Volume (cm3);Prix unitaire (EUR/cm3);Prix ligne (EUR)\n");

    for (filename, volume_cm3, material_name, price_per_cm3) in &model_rows {
        let vol = volume_cm3.unwrap_or(0.0);
        let price = price_per_cm3.unwrap_or(0.0);
        let line_price = vol * price;
        csv.push_str(&format!(
            "{};{};{:.2};{:.3};{:.2}\n",
            filename,
            material_name.as_deref().unwrap_or("Non defini"),
            vol,
            price,
            line_price
        ));
    }

    csv.push('\n');
    let subtotal = breakdown
        .get("subtotal")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    let base_fee = breakdown
        .get("base_fee")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    csv.push_str(&format!("Sous-total materiaux;;;;;;{:.2}\n", subtotal));
    csv.push_str(&format!("Frais de service;;;;;;{:.2}\n", base_fee));
    csv.push_str(&format!("Total;;;;;;{:.2}\n", quote_row.total_price));

    let filename = format!("devis-{}-{}.csv", id_short, date_str);
    let content_disposition = format!("attachment; filename=\"{}\"", filename);

    Ok((
        StatusCode::OK,
        [
            (
                header::CONTENT_TYPE,
                header::HeaderValue::from_static("text/csv; charset=utf-8"),
            ),
            (
                header::CONTENT_DISPOSITION,
                header::HeaderValue::from_str(&content_disposition)
                    .map_err(|e| AppError::Internal(format!("Invalid header value: {e}")))?,
            ),
        ],
        csv,
    ))
}

#[derive(Deserialize)]
pub struct ExportQuery {
    pub format: Option<String>,
}

/// Recalculate a quote with current prices
pub async fn recalculate_quote(
    State(state): State<AppState>,
    jar: CookieJar,
    Path(quote_id): Path<String>,
) -> AppResult<(StatusCode, Json<RecalculateResponse>)> {
    let user = verify_user(&state, &jar).await?;

    // Verify ownership and find the original quote
    let quote_row = persistence::quotes::find_by_id_and_user(&state.pool, &quote_id, &user.id)
        .await?
        .ok_or_else(|| AppError::NotFound("Devis non trouve".to_string()))?;

    if quote_row.status != "generated" {
        return Err(AppError::Validation(
            "Seuls les devis generes peuvent etre recalcules".to_string(),
        ));
    }

    // Get models for this session
    let models = persistence::models::find_by_session(&state.pool, &quote_row.session_id).await?;
    if models.is_empty() {
        return Err(AppError::Internal(
            "Aucun modele dans la session".to_string(),
        ));
    }

    // Build quote items with current prices
    let mut items = Vec::new();
    for model in &models {
        let material_id = match &model.material_id {
            Some(id) => id,
            None => continue,
        };

        let material = persistence::materials::find_by_id(&state.pool, material_id).await?;
        if let Some(material) = material {
            let base_volume = model.volume_cm3.unwrap_or(0.0);
            let support_percentage = model
                .get_support_analysis()
                .map_or(0.0, |s| f64::from(s.estimated_support_material_percentage));
            let support_volume = base_volume * (support_percentage / 100.0);
            let total_volume = base_volume + support_volume;

            let material_cost = crate::business::pricing::calculate_model_price(
                total_volume,
                material.price_per_cm3,
            );

            items.push(crate::business::pricing::QuoteItem {
                model_id: model.id.clone(),
                model_name: model.filename.clone(),
                material_id: material.id.clone(),
                material_name: material.name.clone(),
                volume_cm3: total_volume,
                price_per_cm3: material.price_per_cm3,
                material_cost,
            });
        }
    }

    if items.is_empty() {
        return Err(AppError::Internal(
            "Aucun modele avec materiau assigne".to_string(),
        ));
    }

    // Generate new breakdown
    let breakdown = crate::business::pricing::generate_quote_breakdown(items);

    // Create new quote record
    let new_quote_id = Ulid::new().to_string();
    let now = chrono::Utc::now().naive_utc();
    let breakdown_json = serde_json::to_string(&breakdown)
        .map_err(|e| AppError::Internal(format!("JSON serialization error: {e}")))?;

    persistence::quotes::create(
        &state.pool,
        &new_quote_id,
        &quote_row.session_id,
        breakdown.total,
        &breakdown_json,
        "generated",
        now,
    )
    .await?;

    tracing::info!(
        "User {} recalculated quote {} -> new quote {} (total: {})",
        user.id,
        quote_id,
        new_quote_id,
        breakdown.total
    );

    Ok((
        StatusCode::CREATED,
        Json(RecalculateResponse {
            new_quote_id: new_quote_id.clone(),
            old_quote_id: quote_id,
            session_id: quote_row.session_id,
            total_price: breakdown.total,
            status: "generated".to_string(),
            created_at: now.to_string(),
        }),
    ))
}

#[derive(Serialize)]
pub struct RecalculateResponse {
    pub new_quote_id: String,
    pub old_quote_id: String,
    pub session_id: String,
    pub total_price: f64,
    pub status: String,
    pub created_at: String,
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

    let (id, session_id, total_price, status, breakdown_str, created_at) =
        row.ok_or_else(|| AppError::NotFound("Devis non trouve".to_string()))?;

    let breakdown: serde_json::Value =
        serde_json::from_str(&breakdown_str).unwrap_or(serde_json::json!({}));

    // Fetch models with their materials for this session
    #[allow(clippy::type_complexity)]
    let model_rows: Vec<(String, String, Option<f64>, Option<String>, Option<f64>)> =
        sqlx::query_as(
            r"
        SELECT um.id, um.filename, um.volume_cm3, m.name, m.price_per_cm3
        FROM uploaded_models um
        LEFT JOIN materials m ON um.material_id = m.id
        WHERE um.session_id = $1
        ",
        )
        .bind(&session_id)
        .fetch_all(pool)
        .await?;

    let models = model_rows
        .into_iter()
        .map(|(id, filename, volume_cm3, material_name, price_per_cm3)| {
            let vol = volume_cm3.unwrap_or(0.0);
            let price = price_per_cm3.unwrap_or(0.0) * vol;
            QuoteDetailModel {
                id,
                filename,
                volume_cm3: vol,
                material_name: material_name.unwrap_or_else(|| "Non defini".to_string()),
                price,
            }
        })
        .collect();

    Ok(QuoteDetailResponse {
        id,
        session_id,
        total_price,
        status,
        breakdown,
        created_at,
        models,
    })
}

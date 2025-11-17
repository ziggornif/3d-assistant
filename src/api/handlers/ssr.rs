use axum::{
    extract::{Form, State},
    response::{Html, IntoResponse, Redirect},
};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use serde::Deserialize;
use tera::Context;

use crate::api::handlers::admin::{AdminMaterialResponse, PricingHistoryEntry, PricingHistoryRow};
use crate::api::middleware::AppError;
use crate::api::routes::AppState;
use crate::models::{QuoteSession, material::Material};
use crate::services::render_template;

/// Render the main index page with SSR data
pub async fn index_page(State(state): State<AppState>) -> Result<Html<String>, AppError> {
    // Create a new session
    let session = QuoteSession::new();
    tracing::info!(
        "Query : {} {} {} {}",
        &session.id,
        &session.created_at,
        &session.expires_at,
        &session.status
    );
    sqlx::query(
        r#"
        INSERT INTO quote_sessions (id, created_at, expires_at, status)
        VALUES ($1, $2, $3, $4)
        "#,
    )
    .bind(&session.id)
    .bind(session.created_at)
    .bind(session.expires_at)
    .bind(&session.status)
    .execute(&state.pool)
    .await?;

    // Fetch active materials
    let materials: Vec<Material> = sqlx::query_as(
        r#"
        SELECT id, service_type_id, name, description, price_per_cm3,
               color, properties, active, created_at, updated_at
        FROM materials
        WHERE active = true
        ORDER BY name
        "#,
    )
    .fetch_all(&state.pool)
    .await?;

    // Serialize materials to JSON for embedding in HTML
    let materials_json = serde_json::to_string(&materials)
        .map_err(|e| AppError::Internal(format!("Failed to serialize materials: {}", e)))?;

    // Build template context
    let mut context = Context::new();
    context.insert("session_id", &session.id);
    context.insert("materials_json", &materials_json);
    context.insert("api_base", "");

    // Render template
    let html = render_template("index.html", &context)
        .map_err(|e| AppError::Internal(format!("Template rendering failed: {}", e)))?;

    tracing::info!("SSR rendered index page with session: {}", session.id);
    Ok(Html(html))
}

/// Render the admin page
pub async fn admin_page(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<Html<String>, AppError> {
    let mut context = Context::new();
    context.insert("api_base", "");
    context.insert("login_error", &Option::<String>::None);

    // Check if user is authenticated via cookie
    let is_authenticated = if let Some(cookie) = jar.get("admin_token") {
        cookie.value() == state.config.admin_token
    } else {
        false
    };

    if is_authenticated {
        // Fetch all materials (admin view)
        let materials: Vec<Material> = sqlx::query_as(
            r#"
            SELECT id, service_type_id, name, description, price_per_cm3,
                   color, properties, active, created_at, updated_at
            FROM materials
            ORDER BY name
            "#,
        )
        .fetch_all(&state.pool)
        .await?;

        let admin_materials: Vec<AdminMaterialResponse> =
            materials.into_iter().map(Into::into).collect();
        let materials_json = serde_json::to_string(&admin_materials)
            .map_err(|e| AppError::Internal(format!("Failed to serialize materials: {}", e)))?;

        // Fetch pricing history
        let entries: Vec<PricingHistoryRow> = sqlx::query_as(
            r#"SELECT ph.id, ph.material_id, ph.old_price, ph.new_price, ph.changed_by, ph.changed_at, m.name
            FROM pricing_history ph JOIN materials m ON ph.material_id = m.id ORDER BY ph.changed_at DESC LIMIT 100"#,
        )
        .fetch_all(&state.pool)
        .await?;

        let pricing_history: Vec<PricingHistoryEntry> = entries
            .into_iter()
            .map(
                |(id, material_id, old_price, new_price, changed_by, changed_at, material_name)| {
                    PricingHistoryEntry {
                        id,
                        material_id,
                        material_name,
                        old_price,
                        new_price,
                        changed_by,
                        changed_at,
                    }
                },
            )
            .collect();

        let pricing_history_json = serde_json::to_string(&pricing_history)
            .map_err(|e| AppError::Internal(format!("Failed to serialize history: {}", e)))?;

        context.insert("authenticated", &true);
        context.insert("materials", &admin_materials);
        context.insert("materials_json", &materials_json);
        context.insert("pricing_history", &pricing_history);
        context.insert("pricing_history_json", &pricing_history_json);
    } else {
        context.insert("authenticated", &false);
        context.insert("materials", &Vec::<AdminMaterialResponse>::new());
        context.insert("materials_json", "[]");
        context.insert("pricing_history", &Vec::<PricingHistoryEntry>::new());
        context.insert("pricing_history_json", "[]");
    }

    let html = render_template("admin.html", &context)
        .map_err(|e| AppError::Internal(format!("Template rendering failed: {}", e)))?;

    tracing::info!(
        "SSR rendered admin page (authenticated: {})",
        is_authenticated
    );
    Ok(Html(html))
}

#[derive(Deserialize)]
pub struct LoginForm {
    token: String,
}

/// Handle admin login form submission
pub async fn admin_login(
    State(state): State<AppState>,
    jar: CookieJar,
    Form(form): Form<LoginForm>,
) -> (CookieJar, Redirect) {
    if form.token == state.config.admin_token {
        // Set auth cookie with security flags
        let is_production = state.config.is_production();
        let cookie = Cookie::build(("admin_token", form.token))
            .path("/")
            .http_only(true) // Prevent XSS access
            .secure(is_production) // HTTPS only in production
            .same_site(SameSite::Strict) // CSRF protection
            .max_age(time::Duration::hours(24))
            .build();

        tracing::info!("Admin login successful");
        (jar.add(cookie), Redirect::to("/admin"))
    } else {
        tracing::warn!("Admin login failed: invalid token");
        // Redirect back to admin with error indicator
        (jar, Redirect::to("/admin?error=invalid_token"))
    }
}

/// Handle admin logout
pub async fn admin_logout(jar: CookieJar) -> impl IntoResponse {
    // Remove auth cookie with same security settings
    let cookie = Cookie::build(("admin_token", ""))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Strict)
        .max_age(time::Duration::seconds(0))
        .build();

    tracing::info!("Admin logout");
    (jar.remove(cookie), Redirect::to("/admin"))
}

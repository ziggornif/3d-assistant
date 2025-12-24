use axum::{
    extract::{Form, State},
    response::{Html, IntoResponse, Redirect},
};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use serde::Deserialize;
use tera::Context;

use crate::api::handlers::admin::{AdminMaterialResponse, PricingHistoryEntry};
use crate::api::middleware::AppError;
use crate::api::routes::AppState;
use crate::business::render_template;
use crate::models::QuoteSession;
use crate::persistence;

/// Render the main index page with SSR data
pub async fn index_page(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<(CookieJar, Html<String>), AppError> {
    let mut session: Option<QuoteSession> = None;
    let mut session_id_to_set: Option<String> = None;
    if let Some(cookie) = jar.get("session_id") {
        let sid = cookie.value();
        if let Ok(Some(db_session)) = persistence::sessions::find_by_id(&state.pool, sid).await
            && db_session.expires_at > chrono::Utc::now().naive_utc()
        {
            session = Some(db_session);
        }
    }

    if session.is_none() {
        let new_session = QuoteSession::new();
        persistence::sessions::create(
            &state.pool,
            &new_session.id,
            new_session.created_at,
            new_session.expires_at,
            &new_session.status,
        )
        .await?;
        session_id_to_set = Some(new_session.id.clone());
        session = Some(new_session);
    }
    let session = session.expect("Session should be set");

    // Fetch active materials
    let materials = persistence::materials::list_all_active(&state.pool).await?;

    // Serialize materials to JSON for embedding in HTML
    let materials_json = serde_json::to_string(&materials)
        .map_err(|e| AppError::Internal(format!("Failed to serialize materials: {e}")))?;

    // Build template context
    let mut context = Context::new();
    context.insert("session_id", &session.id);
    context.insert("materials_json", &materials_json);
    context.insert("api_base", "");

    // Render template
    let html = render_template("index.html", &context)
        .map_err(|e| AppError::Internal(format!("Template rendering failed: {e}")))?;

    tracing::info!("SSR rendered index page with session: {}", session.id);

    if let Some(sid) = session_id_to_set {
        let is_production = state.config.is_production();
        let cookie = Cookie::build(("session_id", sid))
            .path("/")
            .http_only(true)
            .secure(is_production)
            .same_site(SameSite::Lax)
            .max_age(time::Duration::hours(24))
            .build();
        Ok((jar.add(cookie), Html(html)))
    } else {
        Ok((jar, Html(html)))
    }
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
        let materials = persistence::materials::list_all(&state.pool).await?;

        let admin_materials: Vec<AdminMaterialResponse> =
            materials.into_iter().map(Into::into).collect();
        let materials_json = serde_json::to_string(&admin_materials)
            .map_err(|e| AppError::Internal(format!("Failed to serialize materials: {e}")))?;

        // Fetch pricing history
        let entries = persistence::admin::get_pricing_history(&state.pool).await?;

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
            .map_err(|e| AppError::Internal(format!("Failed to serialize history: {e}")))?;

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
        .map_err(|e| AppError::Internal(format!("Template rendering failed: {e}")))?;

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

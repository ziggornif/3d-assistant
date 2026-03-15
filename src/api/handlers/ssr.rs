use axum::{
    extract::{Form, Path, Query, State},
    response::{Html, IntoResponse, Redirect},
};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use serde::Deserialize;
use tera::Context;

use crate::api::handlers::admin::{AdminMaterialResponse, PricingHistoryEntry};
use crate::api::middleware::AppError;
use crate::api::routes::AppState;
use crate::business::{AuthService, render_template};
use crate::models::QuoteSession;
use crate::persistence;

#[derive(Deserialize)]
pub struct IndexQuery {
    pub session: Option<String>,
}

/// Render the main index page with SSR data
pub async fn index_page(
    State(state): State<AppState>,
    jar: CookieJar,
    Query(query): Query<IndexQuery>,
) -> Result<(CookieJar, Html<String>), AppError> {
    let mut session: Option<QuoteSession> = None;
    let mut session_id_to_set: Option<String> = None;

    // Priority 1: ?session= query param (e.g. resuming a draft)
    if let Some(ref sid) = query.session
        && let Ok(Some(db_session)) = persistence::sessions::find_by_id(&state.pool, sid).await
        && db_session.expires_at > chrono::Utc::now().naive_utc()
    {
        session_id_to_set = Some(db_session.id.clone());
        session = Some(db_session);
    }

    // Priority 2: session_id cookie
    if session.is_none()
        && let Some(cookie) = jar.get("session_id")
    {
        let sid = cookie.value();
        if let Ok(Some(db_session)) = persistence::sessions::find_by_id(&state.pool, sid).await
            && db_session.expires_at > chrono::Utc::now().naive_utc()
        {
            session = Some(db_session);
        }
    }

    if session.is_none() {
        // Check user authentication to create the right session type
        let authenticated_user = check_user_auth(&state, &jar).await;
        let new_session = if let Some(ref user) = authenticated_user {
            let s = QuoteSession::new_authenticated(user.id.clone());
            persistence::sessions::create_authenticated(
                &state.pool,
                &s.id,
                &user.id,
                s.created_at,
                s.expires_at,
                &s.status,
            )
            .await?;
            s
        } else {
            let s = QuoteSession::new();
            persistence::sessions::create(
                &state.pool,
                &s.id,
                s.created_at,
                s.expires_at,
                &s.status,
            )
            .await?;
            s
        };
        session_id_to_set = Some(new_session.id.clone());
        session = Some(new_session);
    }
    let session = session.expect("Session should be set");

    // Fetch active materials
    let materials = persistence::materials::list_all_active(&state.pool).await?;

    // Serialize materials to JSON for embedding in HTML
    let materials_json = serde_json::to_string(&materials)
        .map_err(|e| AppError::Internal(format!("Failed to serialize materials: {e}")))?;

    // Check user authentication
    let authenticated_user = check_user_auth(&state, &jar).await;
    let is_demo_mode = authenticated_user.is_none();

    // Build template context
    let mut context = Context::new();
    context.insert("session_id", &session.id);
    context.insert("materials_json", &materials_json);
    context.insert("api_base", "");
    context.insert("demo_mode", &is_demo_mode);

    if let Some(ref user) = authenticated_user {
        context.insert("authenticated_user", &true);
        context.insert("user_display_name", &user.display_name);
        context.insert("user_email", &user.email);
    } else {
        context.insert("authenticated_user", &false);
        context.insert("user_display_name", &"");
        context.insert("user_email", &"");
    }

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

/// Render the login page
pub async fn login_page() -> Result<Html<String>, AppError> {
    let context = Context::new();
    let html = render_template("login.html", &context)
        .map_err(|e| AppError::Internal(format!("Template rendering failed: {e}")))?;

    Ok(Html(html))
}

/// Render the register page
pub async fn register_page() -> Result<Html<String>, AppError> {
    let context = Context::new();
    let html = render_template("register.html", &context)
        .map_err(|e| AppError::Internal(format!("Template rendering failed: {e}")))?;

    Ok(Html(html))
}

/// Render the my-quotes page (requires auth, redirects to login if not)
pub async fn my_quotes_page(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<axum::response::Response, AppError> {
    let authenticated_user = check_user_auth(&state, &jar).await;

    if authenticated_user.is_none() {
        return Ok(Redirect::to("/login").into_response());
    }

    let user = authenticated_user.expect("User should be set");

    let mut context = Context::new();
    context.insert("authenticated_user", &true);
    context.insert("user_display_name", &user.display_name);
    context.insert("user_email", &user.email);
    context.insert("user_id", &user.id);
    context.insert("api_base", "");

    let html = render_template("my-quotes.html", &context)
        .map_err(|e| AppError::Internal(format!("Template rendering failed: {e}")))?;

    Ok(Html(html).into_response())
}

/// Render the quote detail page (requires auth)
pub async fn quote_detail_page(
    State(state): State<AppState>,
    jar: CookieJar,
    Path(quote_id): Path<String>,
) -> Result<axum::response::Response, AppError> {
    let authenticated_user = check_user_auth(&state, &jar).await;

    if authenticated_user.is_none() {
        return Ok(Redirect::to("/login").into_response());
    }

    let user = authenticated_user.expect("User should be set");

    let mut context = Context::new();
    context.insert("authenticated_user", &true);
    context.insert("user_display_name", &user.display_name);
    context.insert("quote_id", &quote_id);
    context.insert("api_base", "");

    let html = render_template("quote-detail.html", &context)
        .map_err(|e| AppError::Internal(format!("Template rendering failed: {e}")))?;

    Ok(Html(html).into_response())
}

/// Helper: check user authentication from cookie
async fn check_user_auth(state: &AppState, jar: &CookieJar) -> Option<crate::models::User> {
    let cookie = jar.get("user_session")?;
    let auth_service = AuthService::new(state.pool.clone());
    auth_service.verify_session(cookie.value()).await.ok()
}

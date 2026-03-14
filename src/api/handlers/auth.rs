use axum::{
    Json,
    extract::State,
    http::StatusCode,
};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use serde::{Deserialize, Serialize};

use crate::api::middleware::{AppError, AppResult};
use crate::api::routes::AppState;
use crate::business::auth::{AuthError, AuthService};

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub display_name: String,
}

#[derive(Serialize)]
pub struct RegisterResponse {
    pub user_id: String,
    pub email: String,
    pub display_name: String,
    pub status: String,
    pub message: String,
}

/// Register a new user account
pub async fn register(
    State(state): State<AppState>,
    Json(body): Json<RegisterRequest>,
) -> AppResult<(StatusCode, Json<RegisterResponse>)> {
    let auth_service = AuthService::new(state.pool.clone());

    let user = auth_service
        .register(&body.email, &body.password, &body.display_name)
        .await
        .map_err(|e| match e {
            AuthError::EmailAlreadyExists => {
                AppError::Conflict("Un compte existe deja avec cet email".to_string())
            }
            AuthError::ValidationError(msg) => AppError::Validation(msg),
            AuthError::Internal(msg) => AppError::Internal(msg),
            _ => AppError::Internal(e.to_string()),
        })?;

    Ok((
        StatusCode::CREATED,
        Json(RegisterResponse {
            user_id: user.id,
            email: user.email,
            display_name: user.display_name,
            status: user.status,
            message: "Compte cree, en attente de validation par l'administrateur".to_string(),
        }),
    ))
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub user_id: String,
    pub email: String,
    pub display_name: String,
    pub role: String,
}

/// Login and create a session
pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(body): Json<LoginRequest>,
) -> Result<(CookieJar, Json<LoginResponse>), AppError> {
    let auth_service = AuthService::new(state.pool.clone());

    let (user, token) = auth_service
        .login(&body.email, &body.password)
        .await
        .map_err(|e| match e {
            AuthError::InvalidCredentials => {
                AppError::Unauthorized("Email ou mot de passe incorrect".to_string())
            }
            AuthError::AccountPending => AppError::Forbidden(
                "Compte en attente de validation par l'administrateur".to_string(),
            ),
            AuthError::AccountDisabled => AppError::Forbidden(
                "Votre compte a ete desactive. Contactez l'administrateur.".to_string(),
            ),
            AuthError::Internal(msg) => AppError::Internal(msg),
            _ => AppError::Internal(e.to_string()),
        })?;

    // Set session cookie
    let is_production = state.config.is_production();
    let cookie = Cookie::build(("user_session", token))
        .path("/")
        .http_only(true)
        .secure(is_production)
        .same_site(SameSite::Lax)
        .max_age(time::Duration::days(30))
        .build();

    // Remove anonymous quote session cookie to force creation of an authenticated one
    let remove_session = Cookie::build(("session_id", ""))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .max_age(time::Duration::seconds(0))
        .build();

    Ok((
        jar.add(cookie).remove(remove_session),
        Json(LoginResponse {
            user_id: user.id,
            email: user.email,
            display_name: user.display_name,
            role: user.role,
        }),
    ))
}

#[derive(Serialize)]
pub struct LogoutResponse {
    pub message: String,
}

/// Logout and destroy session
pub async fn logout(
    State(state): State<AppState>,
    jar: CookieJar,
) -> Result<(CookieJar, Json<LogoutResponse>), AppError> {
    // Delete server-side session if token exists
    if let Some(cookie) = jar.get("user_session") {
        let auth_service = AuthService::new(state.pool.clone());
        let _ = auth_service.logout(cookie.value()).await;
    }

    // Remove cookie
    let cookie = Cookie::build(("user_session", ""))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .max_age(time::Duration::seconds(0))
        .build();

    Ok((
        jar.remove(cookie),
        Json(LogoutResponse {
            message: "Deconnecte".to_string(),
        }),
    ))
}

#[derive(Serialize)]
pub struct MeResponse {
    pub user_id: String,
    pub email: String,
    pub display_name: String,
    pub role: String,
    pub status: String,
}

/// Get current authenticated user info
pub async fn me(
    State(state): State<AppState>,
    jar: CookieJar,
) -> AppResult<Json<MeResponse>> {
    let token = jar
        .get("user_session")
        .ok_or_else(|| AppError::Unauthorized("Non authentifie".to_string()))?;

    let auth_service = AuthService::new(state.pool.clone());
    let user = auth_service
        .verify_session(token.value())
        .await
        .map_err(|_| AppError::Unauthorized("Session invalide ou expiree".to_string()))?;

    Ok(Json(MeResponse {
        user_id: user.id,
        email: user.email,
        display_name: user.display_name,
        role: user.role,
        status: user.status,
    }))
}

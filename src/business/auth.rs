use crate::db::DbPool;
use crate::models::user::{self, ROLE_USER, STATUS_ACTIVE, STATUS_PENDING, User};
use crate::persistence;
use anyhow::Result;
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use chrono::{Duration, Utc};
use rand::RngCore;

/// Authentication service error types
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Email ou mot de passe incorrect")]
    InvalidCredentials,

    #[error("Compte en attente de validation par l'administrateur")]
    AccountPending,

    #[error("Votre compte a ete desactive. Contactez l'administrateur.")]
    AccountDisabled,

    #[error("Un compte existe deja avec cet email")]
    EmailAlreadyExists,

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Session invalide ou expiree")]
    InvalidSession,

    #[error("Internal error: {0}")]
    Internal(String),
}

/// Authentication service
pub struct AuthService {
    pool: DbPool,
}

impl AuthService {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    /// Register a new user account
    pub async fn register(
        &self,
        email: &str,
        password: &str,
        display_name: &str,
    ) -> Result<User, AuthError> {
        // Validate inputs
        let email = email.trim().to_lowercase();
        user::validate_email(&email).map_err(AuthError::ValidationError)?;
        user::validate_password(password).map_err(AuthError::ValidationError)?;
        user::validate_display_name(display_name).map_err(AuthError::ValidationError)?;

        // Check email uniqueness
        let existing = persistence::users::find_by_email(&self.pool, &email)
            .await
            .map_err(|e| AuthError::Internal(e.to_string()))?;

        if existing.is_some() {
            return Err(AuthError::EmailAlreadyExists);
        }

        // Hash password with Argon2id
        let password_hash = hash_password(password)?;

        // Create user
        let id = ulid::Ulid::new().to_string();
        let now = Utc::now().naive_utc();

        persistence::users::create(
            &self.pool,
            &id,
            &email,
            &password_hash,
            display_name.trim(),
            STATUS_PENDING,
            ROLE_USER,
            now,
        )
        .await
        .map_err(|e| AuthError::Internal(e.to_string()))?;

        // Return the created user
        let user = persistence::users::find_by_id(&self.pool, &id)
            .await
            .map_err(|e| AuthError::Internal(e.to_string()))?
            .ok_or_else(|| AuthError::Internal("User not found after creation".to_string()))?;

        tracing::info!("New user registered: {} ({})", email, id);

        Ok(user)
    }

    /// Authenticate a user and create a session
    pub async fn login(&self, email: &str, password: &str) -> Result<(User, String), AuthError> {
        let email = email.trim().to_lowercase();

        // Find user by email
        let user = persistence::users::find_by_email(&self.pool, &email)
            .await
            .map_err(|e| AuthError::Internal(e.to_string()))?;

        // Always verify password even if user doesn't exist (constant-time protection)
        let user = match user {
            Some(u) => {
                verify_password(password, &u.password_hash)?;
                u
            }
            None => {
                // Hash a dummy password to prevent timing attacks
                let _ = hash_password(password);
                return Err(AuthError::InvalidCredentials);
            }
        };

        // Check account status
        match user.status.as_str() {
            STATUS_ACTIVE => {} // OK
            STATUS_PENDING => return Err(AuthError::AccountPending),
            _ => return Err(AuthError::AccountDisabled),
        }

        // Generate session token
        let token = generate_session_token();
        let now = Utc::now().naive_utc();
        let expires = (Utc::now() + Duration::days(30)).naive_utc();

        persistence::user_sessions::create(&self.pool, &token, &user.id, now, expires)
            .await
            .map_err(|e| AuthError::Internal(e.to_string()))?;

        tracing::info!("User logged in: {} ({})", user.email, user.id);

        Ok((user, token))
    }

    /// Verify a session token and return the user
    pub async fn verify_session(&self, token: &str) -> Result<User, AuthError> {
        let session = persistence::user_sessions::find_by_token(&self.pool, token)
            .await
            .map_err(|e| AuthError::Internal(e.to_string()))?
            .ok_or(AuthError::InvalidSession)?;

        if session.is_expired() {
            // Clean up expired session
            let _ = persistence::user_sessions::delete_by_token(&self.pool, token).await;
            return Err(AuthError::InvalidSession);
        }

        let user = persistence::users::find_by_id(&self.pool, &session.user_id)
            .await
            .map_err(|e| AuthError::Internal(e.to_string()))?
            .ok_or(AuthError::InvalidSession)?;

        // Check user is still active
        if !user.is_active() {
            return Err(AuthError::AccountDisabled);
        }

        Ok(user)
    }

    /// Logout: delete the session token
    pub async fn logout(&self, token: &str) -> Result<(), AuthError> {
        persistence::user_sessions::delete_by_token(&self.pool, token)
            .await
            .map_err(|e| AuthError::Internal(e.to_string()))?;

        tracing::info!("User session deleted");

        Ok(())
    }
}

/// Hash a password using Argon2id with OWASP recommended parameters
fn hash_password(password: &str) -> Result<String, AuthError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default(); // Uses Argon2id by default with recommended params

    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| AuthError::Internal(format!("Password hashing failed: {}", e)))?;

    Ok(hash.to_string())
}

/// Verify a password against its hash
fn verify_password(password: &str, hash: &str) -> Result<(), AuthError> {
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|_| AuthError::Internal("Invalid hash format".to_string()))?;

    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .map_err(|_| AuthError::InvalidCredentials)
}

/// Generate a cryptographically secure session token
fn generate_session_token() -> String {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    base64::Engine::encode(&base64::engine::general_purpose::URL_SAFE_NO_PAD, bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_and_verify_password() {
        let password = "SecurePass123";
        let hash = hash_password(password).expect("Hashing should succeed");

        assert!(verify_password(password, &hash).is_ok());
        assert!(verify_password("WrongPassword1", &hash).is_err());
    }

    #[test]
    fn test_hash_password_produces_different_hashes() {
        let password = "SecurePass123";
        let hash1 = hash_password(password).expect("Hashing should succeed");
        let hash2 = hash_password(password).expect("Hashing should succeed");

        // Different salts should produce different hashes
        assert_ne!(hash1, hash2);

        // But both should verify correctly
        assert!(verify_password(password, &hash1).is_ok());
        assert!(verify_password(password, &hash2).is_ok());
    }

    #[test]
    fn test_generate_session_token_uniqueness() {
        let token1 = generate_session_token();
        let token2 = generate_session_token();

        assert_ne!(token1, token2);
        assert!(!token1.is_empty());
        // 32 bytes in URL-safe base64 without padding = 43 characters
        assert_eq!(token1.len(), 43);
    }

    #[test]
    fn test_verify_password_invalid_hash() {
        let result = verify_password("password", "not-a-valid-hash");
        assert!(result.is_err());
    }
}

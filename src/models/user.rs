use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// User account status
pub const STATUS_PENDING: &str = "pending";
pub const STATUS_ACTIVE: &str = "active";
pub const STATUS_DISABLED: &str = "disabled";
pub const STATUS_REJECTED: &str = "rejected";

/// User roles
pub const ROLE_USER: &str = "user";
#[allow(dead_code)]
pub const ROLE_ADMIN: &str = "admin";

/// Represents a registered user
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub display_name: String,
    pub status: String,
    pub role: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl User {
    /// Check if the user account is active
    pub fn is_active(&self) -> bool {
        self.status == STATUS_ACTIVE
    }

    /// Check if the user account is pending validation
    #[allow(dead_code)]
    pub fn is_pending(&self) -> bool {
        self.status == STATUS_PENDING
    }

    /// Check if the user account is disabled
    #[allow(dead_code)]
    pub fn is_disabled(&self) -> bool {
        self.status == STATUS_DISABLED
    }

    /// Check if the user is an admin
    #[allow(dead_code)]
    pub fn is_admin(&self) -> bool {
        self.role == ROLE_ADMIN
    }
}

/// Represents an authenticated user session
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserSession {
    pub token: String,
    pub user_id: String,
    pub created_at: NaiveDateTime,
    pub expires_at: NaiveDateTime,
}

impl UserSession {
    /// Check if the session is expired
    pub fn is_expired(&self) -> bool {
        self.expires_at < chrono::Utc::now().naive_utc()
    }
}

/// Validate password strength
/// Returns Ok(()) if valid, Err(message) if not
pub fn validate_password(password: &str) -> Result<(), String> {
    if password.len() < 8 {
        return Err("Le mot de passe doit contenir au moins 8 caracteres".to_string());
    }

    if !password.chars().any(|c| c.is_uppercase()) {
        return Err("Le mot de passe doit contenir au moins une majuscule".to_string());
    }

    if !password.chars().any(|c| c.is_ascii_digit()) {
        return Err("Le mot de passe doit contenir au moins un chiffre".to_string());
    }

    Ok(())
}

/// Validate email format (basic validation)
pub fn validate_email(email: &str) -> Result<(), String> {
    let trimmed = email.trim();

    if trimmed.is_empty() {
        return Err("L'email ne peut pas etre vide".to_string());
    }

    if trimmed.len() > 254 {
        return Err("L'email est trop long".to_string());
    }

    // Basic email validation: must contain exactly one @, with text before and after
    let parts: Vec<&str> = trimmed.split('@').collect();
    if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
        return Err("Format d'email invalide".to_string());
    }

    // Domain must contain at least one dot
    if !parts[1].contains('.') {
        return Err("Format d'email invalide".to_string());
    }

    Ok(())
}

/// Validate display name
pub fn validate_display_name(name: &str) -> Result<(), String> {
    let trimmed = name.trim();

    if trimmed.is_empty() {
        return Err("Le nom ne peut pas etre vide".to_string());
    }

    if trimmed.len() > 100 {
        return Err("Le nom est trop long (max 100 caracteres)".to_string());
    }

    Ok(())
}

/// Validate that a status transition is allowed
pub fn validate_status_transition(new_status: &str) -> Result<(), String> {
    match new_status {
        STATUS_ACTIVE | STATUS_DISABLED | STATUS_REJECTED => Ok(()),
        _ => Err(format!("Statut invalide: {}", new_status)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_password_valid() {
        assert!(validate_password("SecurePass1").is_ok());
        assert!(validate_password("MyP@ss123").is_ok());
        assert!(validate_password("ABCDEFG1h").is_ok());
    }

    #[test]
    fn test_validate_password_too_short() {
        let result = validate_password("Short1");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("8 caracteres"));
    }

    #[test]
    fn test_validate_password_no_uppercase() {
        let result = validate_password("nouppercase1");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("majuscule"));
    }

    #[test]
    fn test_validate_password_no_digit() {
        let result = validate_password("NoDigitHere");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("chiffre"));
    }

    #[test]
    fn test_validate_email_valid() {
        assert!(validate_email("user@example.com").is_ok());
        assert!(validate_email("test@sub.domain.org").is_ok());
        assert!(validate_email("name+tag@company.io").is_ok());
    }

    #[test]
    fn test_validate_email_empty() {
        assert!(validate_email("").is_err());
        assert!(validate_email("   ").is_err());
    }

    #[test]
    fn test_validate_email_no_at() {
        assert!(validate_email("noatsign.com").is_err());
    }

    #[test]
    fn test_validate_email_no_domain_dot() {
        assert!(validate_email("user@nodot").is_err());
    }

    #[test]
    fn test_validate_email_multiple_at() {
        assert!(validate_email("user@@domain.com").is_err());
    }

    #[test]
    fn test_validate_display_name_valid() {
        assert!(validate_display_name("Jean Dupont").is_ok());
        assert!(validate_display_name("A").is_ok());
    }

    #[test]
    fn test_validate_display_name_empty() {
        assert!(validate_display_name("").is_err());
        assert!(validate_display_name("   ").is_err());
    }

    #[test]
    fn test_validate_display_name_too_long() {
        let long_name = "a".repeat(101);
        assert!(validate_display_name(&long_name).is_err());
    }

    #[test]
    fn test_validate_status_transition_valid() {
        assert!(validate_status_transition("active").is_ok());
        assert!(validate_status_transition("disabled").is_ok());
        assert!(validate_status_transition("rejected").is_ok());
    }

    #[test]
    fn test_validate_status_transition_invalid() {
        assert!(validate_status_transition("pending").is_err());
        assert!(validate_status_transition("unknown").is_err());
    }

    #[test]
    fn test_user_is_active() {
        let user = create_test_user(STATUS_ACTIVE);
        assert!(user.is_active());
        assert!(!user.is_pending());
        assert!(!user.is_disabled());
    }

    #[test]
    fn test_user_is_pending() {
        let user = create_test_user(STATUS_PENDING);
        assert!(user.is_pending());
        assert!(!user.is_active());
    }

    #[test]
    fn test_user_is_disabled() {
        let user = create_test_user(STATUS_DISABLED);
        assert!(user.is_disabled());
        assert!(!user.is_active());
    }

    fn create_test_user(status: &str) -> User {
        let now = chrono::Utc::now().naive_utc();
        User {
            id: "test-id".to_string(),
            email: "test@example.com".to_string(),
            password_hash: "hash".to_string(),
            display_name: "Test User".to_string(),
            status: status.to_string(),
            role: ROLE_USER.to_string(),
            created_at: now,
            updated_at: now,
        }
    }
}

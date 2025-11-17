use std::path::Path;

/// Sanitize a filename to prevent directory traversal and other attacks
pub fn sanitize_filename(filename: &str) -> String {
    // Extract just the filename without any path components
    let name = Path::new(filename)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unnamed");

    // Remove or replace dangerous characters
    let sanitized: String = name
        .chars()
        .filter(|c| {
            c.is_alphanumeric()
                || *c == '.'
                || *c == '-'
                || *c == '_'
                || *c == ' '
        })
        .collect();

    // Ensure it's not empty and doesn't start with a dot (hidden file)
    let sanitized = sanitized.trim();
    if sanitized.is_empty() || sanitized == "." || sanitized == ".." {
        "unnamed".to_string()
    } else if sanitized.starts_with('.') {
        format!("_{}", &sanitized[1..])
    } else {
        sanitized.to_string()
    }
}

/// Validate and sanitize a material name
#[allow(dead_code)]
pub fn sanitize_material_name(name: &str) -> Result<String, String> {
    let trimmed = name.trim();

    if trimmed.is_empty() {
        return Err("Material name cannot be empty".to_string());
    }

    if trimmed.len() > 100 {
        return Err("Material name too long (max 100 characters)".to_string());
    }

    // Allow letters, numbers, spaces, and some punctuation
    let sanitized: String = trimmed
        .chars()
        .filter(|c| {
            c.is_alphanumeric()
                || c.is_whitespace()
                || *c == '-'
                || *c == '_'
                || *c == '('
                || *c == ')'
                || *c == 'é'
                || *c == 'è'
                || *c == 'ê'
                || *c == 'à'
                || *c == 'ù'
                || *c == 'ô'
                || *c == 'î'
                || *c == 'ç'
        })
        .collect();

    Ok(sanitized)
}

/// Validate a session ID format (ULID)
#[allow(dead_code)]
pub fn validate_session_id(id: &str) -> bool {
    // ULID is 26 characters, uppercase alphanumeric (Crockford's Base32: 0-9, A-Z)
    id.len() == 26 && id.chars().all(|c| c.is_ascii_digit() || (c.is_ascii_alphabetic() && c.is_ascii_uppercase()))
}

/// Validate a material ID format
#[allow(dead_code)]
pub fn validate_material_id(id: &str) -> bool {
    // Material IDs can be snake_case identifiers or ULIDs
    if id.is_empty() || id.len() > 50 {
        return false;
    }

    // Allow lowercase alphanumeric and underscores (for predefined IDs)
    // or uppercase alphanumeric (for ULIDs)
    id.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Validate price value
#[allow(dead_code)]
pub fn validate_price(price: f64) -> Result<f64, String> {
    if price.is_nan() || price.is_infinite() {
        return Err("Price must be a valid number".to_string());
    }

    if price < 0.0 {
        return Err("Price cannot be negative".to_string());
    }

    if price > 10000.0 {
        return Err("Price exceeds maximum allowed value".to_string());
    }

    // Round to 6 decimal places for precision
    Ok((price * 1_000_000.0).round() / 1_000_000.0)
}

/// Validate color hex code
#[allow(dead_code)]
pub fn validate_color(color: &str) -> Result<String, String> {
    let trimmed = color.trim();

    if !trimmed.starts_with('#') {
        return Err("Color must start with #".to_string());
    }

    if trimmed.len() != 7 {
        return Err("Color must be in #RRGGBB format".to_string());
    }

    if !trimmed[1..].chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("Color must contain valid hex characters".to_string());
    }

    Ok(trimmed.to_uppercase())
}

/// Sanitize a description text
#[allow(dead_code)]
pub fn sanitize_description(text: &str) -> Result<String, String> {
    let trimmed = text.trim();

    if trimmed.len() > 1000 {
        return Err("Description too long (max 1000 characters)".to_string());
    }

    // Remove potentially dangerous HTML/script tags
    let sanitized = trimmed
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;");

    Ok(sanitized)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_filename_normal() {
        assert_eq!(sanitize_filename("model.stl"), "model.stl");
    }

    #[test]
    fn test_sanitize_filename_with_path() {
        assert_eq!(sanitize_filename("/etc/passwd"), "passwd");
        assert_eq!(sanitize_filename("../../../etc/passwd"), "passwd");
    }

    #[test]
    fn test_sanitize_filename_dangerous_chars() {
        assert_eq!(sanitize_filename("model<script>.stl"), "modelscript.stl");
    }

    #[test]
    fn test_sanitize_filename_hidden_file() {
        assert_eq!(sanitize_filename(".hidden"), "_hidden");
    }

    #[test]
    fn test_sanitize_filename_empty() {
        assert_eq!(sanitize_filename(""), "unnamed");
    }

    #[test]
    fn test_validate_session_id_valid() {
        assert!(validate_session_id("01JCV8E3X1MFXZ6BNQWRGY9P0D"));
    }

    #[test]
    fn test_validate_session_id_invalid_length() {
        assert!(!validate_session_id("01JCV8E3X1"));
    }

    #[test]
    fn test_validate_session_id_lowercase() {
        assert!(!validate_session_id("01jcv8e3x1mfxz6bnqwrgy9p0d"));
    }

    #[test]
    fn test_validate_material_id_snake_case() {
        assert!(validate_material_id("pla_white"));
        assert!(validate_material_id("abs_black"));
    }

    #[test]
    fn test_validate_material_id_ulid() {
        assert!(validate_material_id("01JCV8E3X1MFXZ6BNQWRGY9P0D"));
    }

    #[test]
    fn test_validate_material_id_empty() {
        assert!(!validate_material_id(""));
    }

    #[test]
    fn test_validate_price_valid() {
        assert!(validate_price(0.05).is_ok());
        assert!(validate_price(100.0).is_ok());
    }

    #[test]
    fn test_validate_price_negative() {
        assert!(validate_price(-1.0).is_err());
    }

    #[test]
    fn test_validate_price_too_high() {
        assert!(validate_price(100000.0).is_err());
    }

    #[test]
    fn test_validate_color_valid() {
        assert_eq!(validate_color("#FF0000").unwrap(), "#FF0000");
        assert_eq!(validate_color("#ffffff").unwrap(), "#FFFFFF");
    }

    #[test]
    fn test_validate_color_invalid_format() {
        assert!(validate_color("FF0000").is_err());
        assert!(validate_color("#FFF").is_err());
        assert!(validate_color("#GGGGGG").is_err());
    }

    #[test]
    fn test_sanitize_description_html() {
        let result = sanitize_description("<script>alert('xss')</script>").unwrap();
        assert!(!result.contains('<'));
        assert!(!result.contains('>'));
    }

    #[test]
    fn test_sanitize_description_too_long() {
        let long_text = "a".repeat(1001);
        assert!(sanitize_description(&long_text).is_err());
    }

    #[test]
    fn test_sanitize_material_name_valid() {
        assert_eq!(sanitize_material_name("PLA Blanc").unwrap(), "PLA Blanc");
        assert_eq!(sanitize_material_name("ABS résistant").unwrap(), "ABS résistant");
    }

    #[test]
    fn test_sanitize_material_name_empty() {
        assert!(sanitize_material_name("").is_err());
    }

    #[test]
    fn test_sanitize_material_name_too_long() {
        let long_name = "a".repeat(101);
        assert!(sanitize_material_name(&long_name).is_err());
    }
}

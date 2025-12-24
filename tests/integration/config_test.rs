//! Integration tests for configuration loading and validation

use quote_service::config::Config;
use std::env;

#[test]
fn test_config_loads_from_defaults() {
    // Clear environment to test defaults
    env::remove_var("DATABASE_URL");
    env::remove_var("ENVIRONMENT");
    env::remove_var("OTEL_EXPORTER_OTLP_ENDPOINT");

    let config = Config::from_env().expect("Should load with defaults");

    assert_eq!(config.environment, "development");
    assert_eq!(config.otel_exporter_otlp_endpoint, "http://localhost:4317");
    assert_eq!(config.otel_service_name, "quote-service");
    assert_eq!(config.port, 3000);
}

#[test]
fn test_config_loads_from_env_vars() {
    env::set_var("ENVIRONMENT", "production");
    env::set_var("DATABASE_URL", "postgres://prod:password@prod-db:5432/quotes");
    env::set_var("ADMIN_TOKEN", "secure-production-token");
    env::set_var("OTEL_EXPORTER_OTLP_ENDPOINT", "http://otel-collector:4317");
    env::set_var("OTEL_SERVICE_NAME", "quote-service-prod");
    env::set_var("PORT", "8080");

    let config = Config::from_env().expect("Should load from env vars");

    assert_eq!(config.environment, "production");
    assert_eq!(config.database_url, "postgres://prod:password@prod-db:5432/quotes");
    assert_eq!(config.admin_token, "secure-production-token");
    assert_eq!(config.otel_exporter_otlp_endpoint, "http://otel-collector:4317");
    assert_eq!(config.otel_service_name, "quote-service-prod");
    assert_eq!(config.port, 8080);

    // Cleanup
    env::remove_var("ENVIRONMENT");
    env::remove_var("DATABASE_URL");
    env::remove_var("ADMIN_TOKEN");
    env::remove_var("OTEL_EXPORTER_OTLP_ENDPOINT");
    env::remove_var("OTEL_SERVICE_NAME");
    env::remove_var("PORT");
}

#[test]
fn test_is_production_detection() {
    env::set_var("ENVIRONMENT", "production");
    let config = Config::from_env().unwrap();
    assert!(config.is_production());

    env::set_var("ENVIRONMENT", "development");
    let config = Config::from_env().unwrap();
    assert!(!config.is_production());

    env::remove_var("ENVIRONMENT");
}

#[test]
fn test_validation_fails_for_default_database_in_production() {
    env::set_var("ENVIRONMENT", "production");
    env::remove_var("DATABASE_URL");
    env::set_var("ADMIN_TOKEN", "secure-token");

    let config = Config::from_env().unwrap();
    let result = config.validate();

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("DATABASE_URL"));

    env::remove_var("ENVIRONMENT");
    env::remove_var("ADMIN_TOKEN");
}

#[test]
fn test_validation_fails_for_default_admin_token_in_production() {
    env::set_var("ENVIRONMENT", "production");
    env::set_var("DATABASE_URL", "postgres://prod:pass@prod-db/quotes");
    env::remove_var("ADMIN_TOKEN");

    let config = Config::from_env().unwrap();
    let result = config.validate();

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("ADMIN_TOKEN"));

    env::remove_var("ENVIRONMENT");
    env::remove_var("DATABASE_URL");
}

#[test]
fn test_validation_passes_in_development() {
    env::set_var("ENVIRONMENT", "development");
    env::remove_var("DATABASE_URL");
    env::remove_var("ADMIN_TOKEN");

    let config = Config::from_env().unwrap();
    let result = config.validate();

    assert!(result.is_ok());

    env::remove_var("ENVIRONMENT");
}

#[test]
fn test_validation_passes_in_production_with_secure_values() {
    env::set_var("ENVIRONMENT", "production");
    env::set_var("DATABASE_URL", "postgres://prod:securepass@prod-db:5432/quotes");
    env::set_var("ADMIN_TOKEN", "very-secure-random-token-12345");

    let config = Config::from_env().unwrap();
    let result = config.validate();

    assert!(result.is_ok());

    env::remove_var("ENVIRONMENT");
    env::remove_var("DATABASE_URL");
    env::remove_var("ADMIN_TOKEN");
}

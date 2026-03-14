//! Integration tests for MCP (Model Context Protocol) implementation
//!
//! These tests verify the MCP workflow using direct business logic testing
//! rather than JSON-RPC protocol testing (which is handled by rmcp library).

use quote_service::{
    business::{SessionService, file_processor},
    config::Config,
    db::{create_pool, run_migrations, seed_data},
    mcp::create_mcp_router,
    persistence::{materials, models as model_persistence},
};
use sqlx::PgPool;
use tempfile::TempDir;

/// Test helper to create test database pool
async fn setup_test_db() -> PgPool {
    // Load from .env file if it exists
    let _ = dotenvy::dotenv();

    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set for integration tests");

    let pool = create_pool(&database_url, 5)
        .await
        .expect("Failed to create test database pool");

    run_migrations(&pool)
        .await
        .expect("Failed to run migrations");
    seed_data(&pool)
        .await
        .expect("Failed to seed data");

    pool
}

/// Test helper to create test config
fn setup_test_config(temp_dir: &TempDir) -> Config {
    Config {
        database_url: std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set for integration tests"),
        host: "127.0.0.1".to_string(),
        port: 8080,
        upload_dir: temp_dir
            .path()
            .join("uploads")
            .to_string_lossy()
            .to_string(),
        static_dir: temp_dir.path().join("static").to_string_lossy().to_string(),
        template_dir: temp_dir
            .path()
            .join("templates")
            .to_string_lossy()
            .to_string(),
        max_file_size_bytes: 100 * 1024 * 1024,
        admin_token: "test-admin-token".to_string(),
        session_expiry_hours: 24,
        environment: "test".to_string(),
    }
}

/// Helper function to create a test session
async fn create_test_session(pool: &PgPool, upload_dir: &str) -> String {
    let session_service = SessionService::new(pool.clone(), upload_dir);
    let session = session_service
        .create_session()
        .await
        .expect("Failed to create session");
    session.id
}

/// Helper function to create minimal valid STL file (binary format)
/// Creates a simple cube with 12 triangles (2 per face)
fn create_minimal_stl() -> Vec<u8> {
    let mut stl = Vec::new();

    // Header (80 bytes)
    stl.extend_from_slice(&[0u8; 80]);

    // Number of triangles (12 triangles for a cube)
    stl.extend_from_slice(&12u32.to_le_bytes());

    // Helper to add a triangle
    let mut add_triangle = |nx: f32, ny: f32, nz: f32, v1: [f32; 3], v2: [f32; 3], v3: [f32; 3]| {
        stl.extend_from_slice(&nx.to_le_bytes());
        stl.extend_from_slice(&ny.to_le_bytes());
        stl.extend_from_slice(&nz.to_le_bytes());
        stl.extend_from_slice(&v1[0].to_le_bytes());
        stl.extend_from_slice(&v1[1].to_le_bytes());
        stl.extend_from_slice(&v1[2].to_le_bytes());
        stl.extend_from_slice(&v2[0].to_le_bytes());
        stl.extend_from_slice(&v2[1].to_le_bytes());
        stl.extend_from_slice(&v2[2].to_le_bytes());
        stl.extend_from_slice(&v3[0].to_le_bytes());
        stl.extend_from_slice(&v3[1].to_le_bytes());
        stl.extend_from_slice(&v3[2].to_le_bytes());
        stl.extend_from_slice(&0u16.to_le_bytes());
    };

    // 10mm cube
    // Front face (z=10)
    add_triangle(
        0.0,
        0.0,
        1.0,
        [0.0, 0.0, 10.0],
        [10.0, 0.0, 10.0],
        [10.0, 10.0, 10.0],
    );
    add_triangle(
        0.0,
        0.0,
        1.0,
        [0.0, 0.0, 10.0],
        [10.0, 10.0, 10.0],
        [0.0, 10.0, 10.0],
    );

    // Back face (z=0)
    add_triangle(
        0.0,
        0.0,
        -1.0,
        [0.0, 0.0, 0.0],
        [0.0, 10.0, 0.0],
        [10.0, 10.0, 0.0],
    );
    add_triangle(
        0.0,
        0.0,
        -1.0,
        [0.0, 0.0, 0.0],
        [10.0, 10.0, 0.0],
        [10.0, 0.0, 0.0],
    );

    // Right face (x=10)
    add_triangle(
        1.0,
        0.0,
        0.0,
        [10.0, 0.0, 0.0],
        [10.0, 10.0, 0.0],
        [10.0, 10.0, 10.0],
    );
    add_triangle(
        1.0,
        0.0,
        0.0,
        [10.0, 0.0, 0.0],
        [10.0, 10.0, 10.0],
        [10.0, 0.0, 10.0],
    );

    // Left face (x=0)
    add_triangle(
        -1.0,
        0.0,
        0.0,
        [0.0, 0.0, 0.0],
        [0.0, 0.0, 10.0],
        [0.0, 10.0, 10.0],
    );
    add_triangle(
        -1.0,
        0.0,
        0.0,
        [0.0, 0.0, 0.0],
        [0.0, 10.0, 10.0],
        [0.0, 10.0, 0.0],
    );

    // Top face (y=10)
    add_triangle(
        0.0,
        1.0,
        0.0,
        [0.0, 10.0, 0.0],
        [0.0, 10.0, 10.0],
        [10.0, 10.0, 10.0],
    );
    add_triangle(
        0.0,
        1.0,
        0.0,
        [0.0, 10.0, 0.0],
        [10.0, 10.0, 10.0],
        [10.0, 10.0, 0.0],
    );

    // Bottom face (y=0)
    add_triangle(
        0.0,
        -1.0,
        0.0,
        [0.0, 0.0, 0.0],
        [10.0, 0.0, 0.0],
        [10.0, 0.0, 10.0],
    );
    add_triangle(
        0.0,
        -1.0,
        0.0,
        [0.0, 0.0, 0.0],
        [10.0, 0.0, 10.0],
        [0.0, 0.0, 10.0],
    );

    stl
}

#[tokio::test]
async fn test_mcp_service_creation() {
    let pool = setup_test_db().await;
    let temp_dir = TempDir::new().unwrap();
    let config = setup_test_config(&temp_dir);

    // Create upload directory
    std::fs::create_dir_all(&config.upload_dir).unwrap();

    // Create MCP router - this should not panic
    let _router = create_mcp_router(
        pool,
        config.upload_dir.clone(),
        config.max_file_size_bytes as usize,
    );

    // If we get here without panicking, the MCP service was created successfully
    // The router is a StreamableHttpService that will handle MCP protocol requests
}

#[tokio::test]
async fn test_e2e_workflow_business_logic() {
    let pool = setup_test_db().await;
    let temp_dir = TempDir::new().unwrap();
    let config = setup_test_config(&temp_dir);

    // Create upload directory
    std::fs::create_dir_all(&config.upload_dir).unwrap();

    // Step 1: Create session
    let session_id = create_test_session(&pool, &config.upload_dir).await;
    assert!(!session_id.is_empty());

    // Step 2: Get materials
    let materials_list = materials::list_all_active(&pool)
        .await
        .expect("Failed to fetch materials");
    assert!(!materials_list.is_empty());
    let material_id = materials_list[0].id.clone();

    // Step 3: Create and save a model file
    let stl_data = create_minimal_stl();
    let model_id = ulid::Ulid::new().to_string();
    let filename = "test_cube.stl";

    // Validate file
    let file_format =
        file_processor::validate_file(&stl_data, filename, config.max_file_size_bytes)
            .expect("File validation failed");
    assert_eq!(file_format, "stl");

    // Save file
    let file_path = std::path::PathBuf::from(&config.upload_dir)
        .join(&session_id)
        .join(format!("{}.{}", model_id, file_format));

    std::fs::create_dir_all(file_path.parent().unwrap()).unwrap();
    std::fs::write(&file_path, &stl_data).unwrap();

    // Process file to extract metadata
    let processed = file_processor::process_stl_file(file_path.to_str().unwrap())
        .expect("STL processing failed");

    assert!(processed.volume_cm3 > 0.0);
    assert!(processed.dimensions_mm.x > 0.0);
    assert!(processed.triangle_count > 0);

    // Step 4: Create model in database
    use quote_service::models::model::CreateModel;
    let dimensions_json = serde_json::to_string(&serde_json::json!({
        "x": processed.dimensions_mm.x,
        "y": processed.dimensions_mm.y,
        "z": processed.dimensions_mm.z,
    }))
    .unwrap();

    model_persistence::create(
        &pool,
        CreateModel {
            id: &model_id,
            session_id: &session_id,
            filename,
            file_format: &file_format,
            file_size_bytes: stl_data.len() as i64,
            volume_cm3: Some(processed.volume_cm3),
            dimensions_mm: Some(&dimensions_json),
            triangle_count: Some(processed.triangle_count),
            material_id: None,
            file_path: file_path.to_str().unwrap(),
            preview_url: "",
            created_at: chrono::Utc::now().naive_utc(),
            support_analysis: None,
        },
    )
    .await
    .expect("Failed to create model");

    // Step 5: Configure model with material
    model_persistence::update_material(&pool, &model_id, &material_id)
        .await
        .expect("Failed to update material");

    // Step 6: Verify model was configured
    let model = model_persistence::find_by_id_and_session(&pool, &model_id, &session_id)
        .await
        .expect("Failed to find model")
        .expect("Model not found");

    assert_eq!(model.material_id, Some(material_id));
    assert!(model.volume_cm3.is_some());
    assert!(model.dimensions_mm.is_some());
}

#[tokio::test]
async fn test_session_cleanup() {
    let pool = setup_test_db().await;
    let temp_dir = TempDir::new().unwrap();
    let config = setup_test_config(&temp_dir);

    std::fs::create_dir_all(&config.upload_dir).unwrap();

    let session_service = SessionService::new(pool.clone(), &config.upload_dir);

    // Create a session
    let session = session_service
        .create_session()
        .await
        .expect("Failed to create session");

    // Create upload directory for this session
    let session_dir = std::path::PathBuf::from(&config.upload_dir).join(&session.id);
    std::fs::create_dir_all(&session_dir).unwrap();
    assert!(session_dir.exists());

    // For now, session is not expired, cleanup should do nothing
    let result = session_service
        .cleanup_expired()
        .await
        .expect("Cleanup failed");

    assert_eq!(result.sessions_deleted, 0);
    assert!(session_dir.exists());
}

#[tokio::test]
async fn test_mcp_authentication_required() {
    use axum::{
        Router,
        body::Body,
        http::{Request, StatusCode},
        middleware,
    };
    use quote_service::api::middleware::mcp_auth;
    use tower::ServiceExt;

    let pool = setup_test_db().await;
    let temp_dir = TempDir::new().unwrap();
    let config = setup_test_config(&temp_dir);

    std::fs::create_dir_all(&config.upload_dir).unwrap();

    // Create router with MCP authentication
    let mcp_service =
        create_mcp_router(pool, config.upload_dir, config.max_file_size_bytes as usize);
    let router = Router::new()
        .route_service("/mcp", mcp_service)
        .layer(middleware::from_fn(mcp_auth));

    // Test 1: Request without authentication should fail
    let request = Request::builder().uri("/mcp").body(Body::empty()).unwrap();

    let response = router.clone().oneshot(request).await.unwrap();
    assert_eq!(
        response.status(),
        StatusCode::UNAUTHORIZED,
        "Request without auth should be rejected"
    );

    // Test 2: Request with invalid token should fail
    let request = Request::builder()
        .uri("/mcp")
        .header("Authorization", "Bearer invalid-token")
        .body(Body::empty())
        .unwrap();

    let response = router.clone().oneshot(request).await.unwrap();
    assert_eq!(
        response.status(),
        StatusCode::UNAUTHORIZED,
        "Request with invalid token should be rejected"
    );

    // Test 3: Request with valid token should succeed (or at least not return 401)
    let mcp_token = std::env::var("MCP_TOKEN").unwrap_or_else(|_| "mcp-secret-token".to_string());
    let request = Request::builder()
        .uri("/mcp")
        .header("Authorization", format!("Bearer {}", mcp_token))
        .body(Body::empty())
        .unwrap();

    let response = router.oneshot(request).await.unwrap();
    assert_ne!(
        response.status(),
        StatusCode::UNAUTHORIZED,
        "Request with valid token should not be rejected for auth reasons"
    );
}

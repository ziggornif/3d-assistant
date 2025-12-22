use crate::models::{model::CreateModel, quote::UploadedModel};
use chrono::NaiveDateTime;
use sqlx::PgPool;

/// Create a new uploaded model
pub async fn create(pool: &PgPool, model: CreateModel<'_>) -> Result<(), sqlx::Error> {
    sqlx::query(
        r"
        INSERT INTO uploaded_models
        (id, session_id, filename, file_format, file_size_bytes, volume_cm3, dimensions_mm, triangle_count, material_id, file_path, preview_url, created_at, support_analysis)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
        ",
    )
    .bind(model.id)
    .bind(model.session_id)
    .bind(model.filename)
    .bind(model.file_format)
    .bind(model.file_size_bytes)
    .bind(model.volume_cm3)
    .bind(model.dimensions_mm)
    .bind(model.triangle_count)
    .bind(model.material_id)
    .bind(model.file_path)
    .bind(model.preview_url)
    .bind(model.created_at)
    .bind(model.support_analysis)
    .execute(pool)
    .await?;

    Ok(())
}

/// Find model by ID and session ID
pub async fn find_by_id_and_session(
    pool: &PgPool,
    model_id: &str,
    session_id: &str,
) -> Result<Option<UploadedModel>, sqlx::Error> {
    sqlx::query_as(
        r"
        SELECT id, session_id, filename, file_format, file_size_bytes, volume_cm3,
               dimensions_mm, triangle_count, material_id, file_path, preview_url, created_at, support_analysis
        FROM uploaded_models
        WHERE id = $1 AND session_id = $2
        ",
    )
    .bind(model_id)
    .bind(session_id)
    .fetch_optional(pool)
    .await
}

/// Find all models for a session
pub async fn find_by_session(
    pool: &PgPool,
    session_id: &str,
) -> Result<Vec<UploadedModel>, sqlx::Error> {
    sqlx::query_as(
        r"
        SELECT id, session_id, filename, file_format, file_size_bytes, volume_cm3,
               dimensions_mm, triangle_count, material_id, file_path, preview_url, created_at, support_analysis
        FROM uploaded_models
        WHERE session_id = $1
        ",
    )
    .bind(session_id)
    .fetch_all(pool)
    .await
}

/// Update model material
pub async fn update_material(
    pool: &PgPool,
    model_id: &str,
    material_id: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE uploaded_models SET material_id = $1 WHERE id = $2")
        .bind(material_id)
        .bind(model_id)
        .execute(pool)
        .await?;

    Ok(())
}

/// Delete model by ID
pub async fn delete(pool: &PgPool, model_id: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM uploaded_models WHERE id = $1")
        .bind(model_id)
        .execute(pool)
        .await?;

    Ok(())
}

/// Delete all models for expired sessions
pub async fn delete_by_expired_sessions(
    pool: &PgPool,
    now: NaiveDateTime,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r"
        DELETE FROM uploaded_models
        WHERE session_id IN (
            SELECT id FROM quote_sessions
            WHERE expires_at < $1
        )
        ",
    )
    .bind(now)
    .execute(pool)
    .await?;

    Ok(())
}

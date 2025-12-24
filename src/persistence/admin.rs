use chrono::NaiveDateTime;
use sqlx::PgPool;

/// Type alias for pricing history query result
pub type PricingHistoryRow = (
    String,
    String,
    Option<f64>,
    f64,
    Option<String>,
    String,
    String,
);

/// Create pricing history entry
pub async fn create_pricing_history(
    pool: &PgPool,
    id: &str,
    material_id: &str,
    old_price: Option<f64>,
    new_price: f64,
    changed_by: &str,
    changed_at: NaiveDateTime,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r"
        INSERT INTO pricing_history (id, material_id, old_price, new_price, changed_by, changed_at)
        VALUES ($1, $2, $3, $4, $5, $6)
        ",
    )
    .bind(id)
    .bind(material_id)
    .bind(old_price)
    .bind(new_price)
    .bind(changed_by)
    .bind(changed_at)
    .execute(pool)
    .await?;

    Ok(())
}

/// Get pricing history with material names
pub async fn get_pricing_history(pool: &PgPool) -> Result<Vec<PricingHistoryRow>, sqlx::Error> {
    sqlx::query_as(
        r"
        SELECT ph.id, ph.material_id, ph.old_price, ph.new_price, ph.changed_by, ph.changed_at, m.name
        FROM pricing_history ph
        JOIN materials m ON ph.material_id = m.id
        ORDER BY ph.changed_at DESC
        LIMIT 100
        ",
    )
    .fetch_all(pool)
    .await
}

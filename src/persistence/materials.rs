use crate::models::material::{CreateMaterial, Material, UpdateMaterial};
use sqlx::PgPool;

/// List all active materials
pub async fn list_all_active(pool: &PgPool) -> Result<Vec<Material>, sqlx::Error> {
    sqlx::query_as(
        r"
        SELECT id, service_type_id, name, description, price_per_cm3,
               color, properties, active, created_at, updated_at
        FROM materials
        WHERE active = true
        ORDER BY name
        ",
    )
    .fetch_all(pool)
    .await
}

/// List active materials by service type
pub async fn list_by_service_type(
    pool: &PgPool,
    service_type: &str,
) -> Result<Vec<Material>, sqlx::Error> {
    sqlx::query_as(
        r"
        SELECT m.id, m.service_type_id, m.name, m.description, m.price_per_cm3,
               m.color, m.properties, m.active, m.created_at, m.updated_at
        FROM materials m
        JOIN service_types st ON m.service_type_id = st.id
        WHERE st.name = $1 AND m.active = true
        ORDER BY m.name
        ",
    )
    .bind(service_type)
    .fetch_all(pool)
    .await
}

/// Find material by ID
pub async fn find_by_id(pool: &PgPool, id: &str) -> Result<Option<Material>, sqlx::Error> {
    sqlx::query_as(
        r"
        SELECT id, service_type_id, name, description, price_per_cm3,
               color, properties, active, created_at, updated_at
        FROM materials
        WHERE id = $1
        ",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

/// Create a new material
pub async fn create(pool: &PgPool, material: CreateMaterial<'_>) -> Result<Material, sqlx::Error> {
    sqlx::query_as(
        r"
        INSERT INTO materials (id, service_type_id, name, description, price_per_cm3, color, properties, active, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, true, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
        RETURNING id, service_type_id, name, description, price_per_cm3, color, properties, active, created_at, updated_at
        ",
    )
    .bind(material.id)
    .bind(material.service_type_id)
    .bind(material.name)
    .bind(material.description)
    .bind(material.price_per_cm3)
    .bind(material.color)
    .bind(material.properties)
    .fetch_one(pool)
    .await
}

/// Update material
pub async fn update(pool: &PgPool, material: UpdateMaterial<'_>) -> Result<Material, sqlx::Error> {
    let mut query = "UPDATE materials SET updated_at = CURRENT_TIMESTAMP".to_string();
    let mut params: Vec<String> = vec![];
    let mut param_count = 1;

    if material.name.is_some() {
        params.push(format!("name = ${}", param_count));
        param_count += 1;
    }
    if material.description.is_some() {
        params.push(format!("description = ${}", param_count));
        param_count += 1;
    }
    if material.price_per_cm3.is_some() {
        params.push(format!("price_per_cm3 = ${}", param_count));
        param_count += 1;
    }
    if material.color.is_some() {
        params.push(format!("color = ${}", param_count));
        param_count += 1;
    }
    if material.properties.is_some() {
        params.push(format!("properties = ${}", param_count));
        param_count += 1;
    }
    if material.active.is_some() {
        params.push(format!("active = ${}", param_count));
        param_count += 1;
    }

    if !params.is_empty() {
        query.push_str(", ");
        query.push_str(&params.join(", "));
    }

    query.push_str(&format!(
        " WHERE id = ${} RETURNING id, service_type_id, name, description, price_per_cm3, color, properties, active, created_at, updated_at",
        param_count
    ));

    let mut q = sqlx::query_as(&query);
    if let Some(n) = material.name {
        q = q.bind(n);
    }
    if let Some(d) = material.description {
        q = q.bind(d);
    }
    if let Some(p) = material.price_per_cm3 {
        q = q.bind(p);
    }
    if let Some(c) = material.color {
        q = q.bind(c);
    }
    if let Some(pr) = material.properties {
        q = q.bind(pr);
    }
    if let Some(a) = material.active {
        q = q.bind(a);
    }
    q = q.bind(material.id);

    q.fetch_one(pool).await
}

/// List all materials (including inactive) - admin only
pub async fn list_all(pool: &PgPool) -> Result<Vec<Material>, sqlx::Error> {
    sqlx::query_as(
        r"
        SELECT id, service_type_id, name, description, price_per_cm3,
               color, properties, active, created_at, updated_at
        FROM materials
        ORDER BY service_type_id, name
        ",
    )
    .fetch_all(pool)
    .await
}

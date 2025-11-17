use anyhow::Result;
use sqlx::{Pool, Postgres, postgres::PgPoolOptions};

pub mod schema;

/// Database connection pool wrapper (PostgreSQL)
pub type DbPool = Pool<Postgres>;

/// Initialize database connection pool
pub async fn init_pool(database_url: &str) -> Result<DbPool> {
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .min_connections(1)
        .acquire_timeout(std::time::Duration::from_secs(30))
        .connect(database_url)
        .await?;

    tracing::info!(
        "Connected to database: {}",
        database_url.split('?').next().unwrap_or(database_url)
    );

    Ok(pool)
}

/// Run database migrations
pub async fn run_migrations(pool: &DbPool) -> Result<()> {
    let migrations_dir = std::path::Path::new("src/db/migrations");

    if !migrations_dir.exists() {
        anyhow::bail!("Migrations directory not found: {:?}", migrations_dir);
    }

    let mut entries: Vec<_> = std::fs::read_dir(migrations_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext == "sql")
                .unwrap_or(false)
        })
        .collect();

    entries.sort_by_key(|e| e.path());

    for entry in entries {
        let sql = std::fs::read_to_string(entry.path())?;
        tracing::info!("Running migration: {:?}", entry.path().file_name());
        // Execute each statement separately (required for PostgreSQL)
        for statement in sql.split(';').filter(|s| !s.trim().is_empty()) {
            sqlx::query(statement).execute(pool).await?;
        }
    }

    Ok(())
}

/// Seed initial data
pub async fn seed_data(pool: &DbPool) -> Result<()> {
    let seed_path = std::path::Path::new("src/db/seed.sql");

    if seed_path.exists() {
        let sql = std::fs::read_to_string(seed_path)?;
        tracing::info!("Running seed data");

        // Execute each statement separately
        for statement in sql.split(';').filter(|s| !s.trim().is_empty()) {
            sqlx::query(statement).execute(pool).await?;
        }
    }

    Ok(())
}

use sqlx::PgPool;
use anyhow::Result;

pub async fn connect(db_url: &str) -> Result<PgPool> {
    let pool = PgPool::connect(db_url).await?;
    Ok(pool)
}

pub async fn ensure_migrations(pool: &PgPool) -> Result<(), sqlx::Error> {
    // Run the simple migration SQL from file path migrations/001_create_tables.sql
    let sql = std::fs::read_to_string("migrations/001_create_tables.sql").expect("migrations file missing");
    sqlx::query(&sql).execute(pool).await?;
    Ok(())
}
//! Civitas database layer.
//!
//! Holds all `SQLx` queries and the migration history. Exposes typed
//! functions to upstream crates; `SQL` strings do not leak past this boundary.
//!
//! Migrations live in `backend/migrations/` and are applied in lexical order
//! by `sqlx migrate run`. Migrations are forward-only — once merged to `main`,
//! a migration is immutable; corrections come as new migrations.
//!
//! Implementation lands in a follow-up session; this is the wiring stub.

#![doc(html_root_url = "https://docs.rs/civitas-db/0.1.0")]

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("database error: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("migration error: {0}")]
    Migrate(#[from] sqlx::migrate::MigrateError),
}

pub type DbResult<T> = std::result::Result<T, DbError>;

/// Connect to Postgres using the configured pool size.
pub async fn connect(database_url: &str, max_connections: u32) -> DbResult<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(max_connections)
        .connect(database_url)
        .await?;
    Ok(pool)
}

/// Apply pending migrations.
pub async fn migrate(pool: &PgPool) -> DbResult<()> {
    sqlx::migrate!("../../migrations").run(pool).await?;
    Ok(())
}

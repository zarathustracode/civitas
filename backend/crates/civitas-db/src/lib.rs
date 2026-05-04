//! Civitas database layer.
//!
//! Holds all `SQLx` queries and the migration history. Exposes typed
//! functions to upstream crates; `SQL` strings do not leak past this boundary.
//!
//! Migrations live in `backend/migrations/` and are applied in lexical order
//! by `sqlx migrate run`. Migrations are forward-only — once merged to `main`,
//! a migration is immutable; corrections come as new migrations.
//!
#![doc(html_root_url = "https://docs.rs/civitas-db/0.1.0")]

use chrono::{DateTime, Utc};
use civitas_types::{AuditLogId, TopicId, UserId};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("database error: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("migration error: {0}")]
    Migrate(#[from] sqlx::migrate::MigrateError),
}

pub type DbResult<T> = std::result::Result<T, DbError>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Topic {
    pub id: TopicId,
    pub slug: String,
    pub name: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NewTopic<'a> {
    pub slug: &'a str,
    pub name: &'a str,
    pub description: &'a str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditLogEntry {
    pub id: AuditLogId,
    pub actor_id: Option<UserId>,
    pub action: String,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewAuditLogEntry<'a> {
    pub actor_id: Option<UserId>,
    pub action: &'a str,
    pub entity_type: &'a str,
    pub entity_id: Uuid,
    pub metadata: Value,
}

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

pub async fn create_topic(pool: &PgPool, input: NewTopic<'_>) -> DbResult<Topic> {
    let id = TopicId::new();
    let row = sqlx::query!(
        r#"
        insert into topics (id, slug, name, description)
        values ($1, $2, $3, $4)
        returning id, slug, name, description, created_at
        "#,
        id.into_inner(),
        input.slug,
        input.name,
        input.description,
    )
    .fetch_one(pool)
    .await?;

    Ok(Topic {
        id: TopicId::from_uuid(row.id),
        slug: row.slug,
        name: row.name,
        description: row.description,
        created_at: row.created_at,
    })
}

pub async fn list_topics(pool: &PgPool) -> DbResult<Vec<Topic>> {
    let rows = sqlx::query!(
        r#"
        select id, slug, name, description, created_at
        from topics
        order by name asc, slug asc
        "#
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| Topic {
            id: TopicId::from_uuid(row.id),
            slug: row.slug,
            name: row.name,
            description: row.description,
            created_at: row.created_at,
        })
        .collect())
}

pub async fn create_audit_log_entry(
    pool: &PgPool,
    input: NewAuditLogEntry<'_>,
) -> DbResult<AuditLogEntry> {
    let id = AuditLogId::new();
    let actor_id = input.actor_id.map(UserId::into_inner);

    let row = sqlx::query!(
        r#"
        insert into audit_log (id, actor_id, action, entity_type, entity_id, metadata)
        values ($1, $2, $3, $4, $5, $6)
        returning id, actor_id, action, entity_type, entity_id, metadata, created_at
        "#,
        id.into_inner(),
        actor_id,
        input.action,
        input.entity_type,
        input.entity_id,
        input.metadata,
    )
    .fetch_one(pool)
    .await?;

    Ok(AuditLogEntry {
        id: AuditLogId::from_uuid(row.id),
        actor_id: row.actor_id.map(UserId::from_uuid),
        action: row.action,
        entity_type: row.entity_type,
        entity_id: row.entity_id,
        metadata: row.metadata,
        created_at: row.created_at,
    })
}

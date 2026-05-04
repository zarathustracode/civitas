//! Topics.
//!
//! Topics are the unit of delegation. They are sparse (a few dozen per
//! deployment) and rarely change. v1 has no edit or delete operations on
//! topics; if a topic needs renaming, that is a future migration with a
//! grace period.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgExecutor, Postgres, Transaction};

use civitas_types::{TopicId, UserId};

use crate::audit::{write_log, Action};
use crate::{DbError, DbResult};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::FromRow)]
pub struct Topic {
    pub id: TopicId,
    pub slug: String,
    pub name: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct NewTopic<'a> {
    pub slug: &'a str,
    pub name: &'a str,
    pub description: &'a str,
}

pub async fn create(
    tx: &mut Transaction<'_, Postgres>,
    actor_id: UserId,
    new: NewTopic<'_>,
) -> DbResult<Topic> {
    let id = TopicId::new();

    let topic = sqlx::query_as!(
        Topic,
        r#"
        insert into topics (id, slug, name, description)
        values ($1, $2, $3, $4)
        returning id as "id: TopicId", slug, name, description, created_at
        "#,
        id.into_inner(),
        new.slug,
        new.name,
        new.description,
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(map_unique_violation_to_slug_taken)?;

    write_log(
        &mut **tx,
        Some(actor_id),
        Action::TopicCreated,
        "topic",
        topic.id.into_inner(),
        None,
    )
    .await?;

    Ok(topic)
}

pub async fn find_by_id<'c, E: PgExecutor<'c>>(conn: E, id: TopicId) -> DbResult<Option<Topic>> {
    let row = sqlx::query_as!(
        Topic,
        r#"
        select id as "id: TopicId", slug, name, description, created_at
        from topics
        where id = $1
        "#,
        id.into_inner(),
    )
    .fetch_optional(conn)
    .await?;
    Ok(row)
}

pub async fn find_by_slug<'c, E: PgExecutor<'c>>(conn: E, slug: &str) -> DbResult<Option<Topic>> {
    let row = sqlx::query_as!(
        Topic,
        r#"
        select id as "id: TopicId", slug, name, description, created_at
        from topics
        where slug = $1
        "#,
        slug,
    )
    .fetch_optional(conn)
    .await?;
    Ok(row)
}

pub async fn list<'c, E: PgExecutor<'c>>(conn: E) -> DbResult<Vec<Topic>> {
    let rows = sqlx::query_as!(
        Topic,
        r#"
        select id as "id: TopicId", slug, name, description, created_at
        from topics
        order by name
        "#,
    )
    .fetch_all(conn)
    .await?;
    Ok(rows)
}

fn map_unique_violation_to_slug_taken(err: sqlx::Error) -> DbError {
    if let sqlx::Error::Database(db_err) = &err {
        if db_err.code().as_deref() == Some("23505")
            && db_err.constraint() == Some("topics_slug_uniq")
        {
            return DbError::SlugAlreadyTaken;
        }
    }
    DbError::from(err)
}

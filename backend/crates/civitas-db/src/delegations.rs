//! Delegations.
//!
//! `create_with_cycle_check` is the canonical creation path. It serializes
//! concurrent delegation modifications on the same topic via a transaction-
//! scoped advisory lock so the cycle check cannot be bypassed by a race.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgExecutor, Postgres, Transaction};

use civitas_core::{
    delegation::{would_create_cycle, CycleCheck, ProposedDelegation},
    DelegationRecord,
};
use civitas_types::{DelegationId, TopicId, UserId};

use crate::audit::{write_log, Action};
use crate::{DbError, DbResult};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DelegationRow {
    pub id: DelegationId,
    pub delegator_id: UserId,
    pub delegate_id: UserId,
    pub topic_id: TopicId,
    pub created_at: DateTime<Utc>,
    pub revoked_at: Option<DateTime<Utc>>,
}

/// Create a new delegation if doing so does not introduce a cycle.
///
/// Holds an advisory transaction lock keyed on `topic_id` so that concurrent
/// delegation creation on the same topic is serialized — preventing the
/// TOCTOU between cycle check and insert.
pub async fn create_with_cycle_check(
    tx: &mut Transaction<'_, Postgres>,
    delegator_id: UserId,
    delegate_id: UserId,
    topic_id: TopicId,
) -> DbResult<DelegationRow> {
    if delegator_id == delegate_id {
        return Err(DbError::DelegationSelf);
    }

    advisory_lock_topic(tx, topic_id).await?;

    let active = load_active_for_topic_inner(&mut **tx, topic_id).await?;

    let proposed = ProposedDelegation {
        delegator_id,
        delegate_id,
        topic_id,
    };
    match would_create_cycle(&active, &proposed) {
        CycleCheck::Acyclic => {}
        CycleCheck::Cyclic => return Err(DbError::DelegationCyclic),
        CycleCheck::SelfDelegation => return Err(DbError::DelegationSelf),
        CycleCheck::DepthExceeded => return Err(DbError::DelegationDepthExceeded),
    }

    let id = DelegationId::new();
    let row = sqlx::query_as!(
        DelegationRow,
        r#"
        insert into delegations (id, delegator_id, delegate_id, topic_id)
        values ($1, $2, $3, $4)
        returning
            id           as "id: DelegationId",
            delegator_id as "delegator_id: UserId",
            delegate_id  as "delegate_id: UserId",
            topic_id     as "topic_id: TopicId",
            created_at,
            revoked_at
        "#,
        id.into_inner(),
        delegator_id.into_inner(),
        delegate_id.into_inner(),
        topic_id.into_inner(),
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(map_unique_violation_to_already_delegating)?;

    let metadata = serde_json::json!({
        "delegator_id": delegator_id.to_string(),
        "delegate_id": delegate_id.to_string(),
        "topic_id": topic_id.to_string(),
    });
    write_log(
        &mut **tx,
        Some(delegator_id),
        Action::DelegationCreated,
        "delegation",
        row.id.into_inner(),
        Some(&metadata),
    )
    .await?;

    Ok(row)
}

/// Revoke a delegation. The delegator is the only legitimate revoker; the
/// caller is responsible for that authorization check.
pub async fn revoke(
    tx: &mut Transaction<'_, Postgres>,
    actor_id: UserId,
    delegation_id: DelegationId,
) -> DbResult<()> {
    let updated = sqlx::query!(
        r#"
        update delegations
        set revoked_at = now()
        where id = $1 and revoked_at is null
        "#,
        delegation_id.into_inner(),
    )
    .execute(&mut **tx)
    .await?;

    if updated.rows_affected() == 0 {
        return Err(DbError::NotFound);
    }

    write_log(
        &mut **tx,
        Some(actor_id),
        Action::DelegationRevoked,
        "delegation",
        delegation_id.into_inner(),
        None,
    )
    .await?;

    Ok(())
}

pub async fn load_active_for_topic<'c, E: PgExecutor<'c>>(
    conn: E,
    topic_id: TopicId,
) -> DbResult<Vec<DelegationRecord>> {
    load_active_for_topic_inner(conn, topic_id).await
}

async fn load_active_for_topic_inner<'c, E: PgExecutor<'c>>(
    conn: E,
    topic_id: TopicId,
) -> DbResult<Vec<DelegationRecord>> {
    let rows = sqlx::query!(
        r#"
        select
            id           as "id: DelegationId",
            delegator_id as "delegator_id: UserId",
            delegate_id  as "delegate_id: UserId",
            topic_id     as "topic_id: TopicId",
            created_at,
            revoked_at
        from delegations
        where topic_id = $1 and revoked_at is null
        "#,
        topic_id.into_inner(),
    )
    .fetch_all(conn)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| DelegationRecord {
            id: r.id,
            delegator_id: r.delegator_id,
            delegate_id: r.delegate_id,
            topic_id: r.topic_id,
            created_at: r.created_at,
            revoked_at: r.revoked_at,
        })
        .collect())
}

/// The active delegation a user has on a topic, if any.
pub async fn find_active_for_user_on_topic<'c, E: PgExecutor<'c>>(
    conn: E,
    delegator_id: UserId,
    topic_id: TopicId,
) -> DbResult<Option<DelegationRow>> {
    let row = sqlx::query!(
        r#"
        select
            id           as "id: DelegationId",
            delegator_id as "delegator_id: UserId",
            delegate_id  as "delegate_id: UserId",
            topic_id     as "topic_id: TopicId",
            created_at,
            revoked_at
        from delegations
        where delegator_id = $1 and topic_id = $2 and revoked_at is null
        "#,
        delegator_id.into_inner(),
        topic_id.into_inner(),
    )
    .fetch_optional(conn)
    .await?;

    Ok(row.map(|r| DelegationRow {
        id: r.id,
        delegator_id: r.delegator_id,
        delegate_id: r.delegate_id,
        topic_id: r.topic_id,
        created_at: r.created_at,
        revoked_at: r.revoked_at,
    }))
}

/// All active delegations a user has authored, across topics.
pub async fn list_active_by_delegator<'c, E: PgExecutor<'c>>(
    conn: E,
    delegator_id: UserId,
) -> DbResult<Vec<DelegationRow>> {
    let rows = sqlx::query!(
        r#"
        select
            id           as "id: DelegationId",
            delegator_id as "delegator_id: UserId",
            delegate_id  as "delegate_id: UserId",
            topic_id     as "topic_id: TopicId",
            created_at,
            revoked_at
        from delegations
        where delegator_id = $1 and revoked_at is null
        order by created_at desc
        "#,
        delegator_id.into_inner(),
    )
    .fetch_all(conn)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| DelegationRow {
            id: r.id,
            delegator_id: r.delegator_id,
            delegate_id: r.delegate_id,
            topic_id: r.topic_id,
            created_at: r.created_at,
            revoked_at: r.revoked_at,
        })
        .collect())
}

/// Take a transaction-scoped advisory lock keyed on the topic. Released
/// automatically at commit/rollback. Effective range: u64 from the lower
/// 8 bytes of the topic UUID, reinterpreted as i64 for Postgres' bigint.
async fn advisory_lock_topic(
    tx: &mut Transaction<'_, Postgres>,
    topic_id: TopicId,
) -> DbResult<()> {
    let bytes = topic_id.into_inner().into_bytes();
    let key = i64::from_be_bytes(bytes[8..16].try_into().expect("uuid is 16 bytes"));
    sqlx::query!("select pg_advisory_xact_lock($1)", key)
        .execute(&mut **tx)
        .await?;
    Ok(())
}

fn map_unique_violation_to_already_delegating(err: sqlx::Error) -> DbError {
    if let sqlx::Error::Database(db_err) = &err {
        if db_err.code().as_deref() == Some("23505")
            && db_err.constraint() == Some("delegations_delegator_topic_active_uniq")
        {
            return DbError::DelegationAlreadyActive;
        }
    }
    DbError::from(err)
}

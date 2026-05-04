//! Audit log writes.
//!
//! Every voting-relevant state change records a row here. Callers compose
//! the audit write into the same transaction as the change so the two are
//! atomic — if the audit row fails, the change is rolled back. All public
//! action codes are listed in `Action`; ad-hoc strings are discouraged but
//! supported via [`write_log_raw`] for cross-cutting cases.

use serde_json::Value as JsonValue;
use sqlx::PgExecutor;
use uuid::Uuid;

use civitas_types::{AuditLogId, UserId};

use crate::DbResult;

/// Stable audit action codes.
///
/// Adding a new variant is a deliberate act — it appears in operator
/// dashboards and external analyses. Prefer a stable string here over an
/// ad-hoc literal at the call site.
#[derive(Debug, Clone, Copy)]
pub enum Action {
    UserRegistered,
    UserEmailVerified,
    UserPhoneVerified,
    UserPasswordChanged,
    UserDeleted,
    SessionCreated,
    SessionRevoked,
    TopicCreated,
    ProposalCreated,
    ProposalStatusChanged,
    VoteCast,
    DelegationCreated,
    DelegationRevoked,
    CommentPosted,
    CommentEdited,
    CommentDeleted,
    CommentHidden,
}

impl Action {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Action::UserRegistered => "user.registered",
            Action::UserEmailVerified => "user.email_verified",
            Action::UserPhoneVerified => "user.phone_verified",
            Action::UserPasswordChanged => "user.password_changed",
            Action::UserDeleted => "user.deleted",
            Action::SessionCreated => "session.created",
            Action::SessionRevoked => "session.revoked",
            Action::TopicCreated => "topic.created",
            Action::ProposalCreated => "proposal.created",
            Action::ProposalStatusChanged => "proposal.status_changed",
            Action::VoteCast => "vote.cast",
            Action::DelegationCreated => "delegation.created",
            Action::DelegationRevoked => "delegation.revoked",
            Action::CommentPosted => "comment.posted",
            Action::CommentEdited => "comment.edited",
            Action::CommentDeleted => "comment.deleted",
            Action::CommentHidden => "comment.hidden",
        }
    }
}

/// Write an audit row using a typed [`Action`] code.
pub async fn write_log<'c, E: PgExecutor<'c>>(
    conn: E,
    actor_id: Option<UserId>,
    action: Action,
    entity_type: &str,
    entity_id: Uuid,
    metadata: Option<&JsonValue>,
) -> DbResult<AuditLogId> {
    write_log_raw(
        conn,
        actor_id,
        action.as_str(),
        entity_type,
        entity_id,
        metadata,
    )
    .await
}

/// Lower-level audit write that takes an arbitrary action string. Use only
/// when [`Action`] does not yet have the variant you need; prefer adding a
/// variant.
pub async fn write_log_raw<'c, E: PgExecutor<'c>>(
    conn: E,
    actor_id: Option<UserId>,
    action: &str,
    entity_type: &str,
    entity_id: Uuid,
    metadata: Option<&JsonValue>,
) -> DbResult<AuditLogId> {
    let id = AuditLogId::new();
    let default_meta = JsonValue::Object(serde_json::Map::new());
    let metadata = metadata.unwrap_or(&default_meta);

    sqlx::query!(
        r#"
        insert into audit_log (id, actor_id, action, entity_type, entity_id, metadata)
        values ($1, $2, $3, $4, $5, $6)
        "#,
        id.into_inner(),
        actor_id.map(UserId::into_inner),
        action,
        entity_type,
        entity_id,
        metadata,
    )
    .execute(conn)
    .await?;

    Ok(id)
}

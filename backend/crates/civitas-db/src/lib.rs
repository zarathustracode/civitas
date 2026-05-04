//! Civitas database layer.
//!
//! Holds all `SQLx` queries and the migration history. Exposes typed
//! functions to upstream crates; `SQL` strings do not leak past this boundary.
//!
//! Migrations live in `backend/migrations/` and are applied in lexical order
//! by `sqlx migrate run`. Migrations are forward-only — once merged to `main`,
//! a migration is immutable; corrections come as new migrations.
//!
//! ## Composition rules
//!
//! - **Reads** take `impl PgExecutor<'_>` so callers can pass a pool, a
//!   connection, or a transaction.
//! - **Writes that touch voting-relevant state** take
//!   `&mut Transaction<'_, Postgres>` and write the audit row in the same
//!   transaction. Callers are responsible for committing.

#![doc(html_root_url = "https://docs.rs/civitas-db/0.1.0")]

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

pub mod audit;
pub mod comments;
pub mod delegations;
pub mod eligibility;
pub mod proposals;
pub mod sessions;
pub mod tokens;
pub mod topics;
pub mod users;
pub mod votes;

#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("database error: {0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("migration error: {0}")]
    Migrate(#[from] sqlx::migrate::MigrateError),

    #[error("not found")]
    NotFound,

    #[error("email already taken")]
    EmailAlreadyTaken,
    #[error("phone already taken")]
    PhoneAlreadyTaken,
    #[error("topic slug already taken")]
    SlugAlreadyTaken,

    #[error("invalid state transition: {from} → {to}")]
    InvalidStateTransition {
        from: &'static str,
        to: &'static str,
    },
    #[error("voting window is required when transitioning to voting")]
    VotingWindowRequired,
    #[error("voting window must have start < end")]
    VotingWindowInvalid,

    #[error("proposal is not in voting status")]
    ProposalNotInVoting,
    #[error("vote cast outside the proposal's voting window")]
    OutsideVotingWindow,

    #[error("delegator and delegate must be different users")]
    DelegationSelf,
    #[error("delegation would close a cycle")]
    DelegationCyclic,
    #[error("delegation chain depth exceeded")]
    DelegationDepthExceeded,
    #[error("user already has an active delegation on this topic")]
    DelegationAlreadyActive,

    #[error("comments are not allowed while the proposal is in {0}")]
    CommentsNotAllowedInStatus(&'static str),
    #[error("parent comment belongs to a different proposal")]
    CommentParentMismatch,
    #[error("a non-empty reason is required")]
    ReasonRequired,
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

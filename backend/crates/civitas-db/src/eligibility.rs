//! Eligibility data loading for tally and authorization.
//!
//! These functions translate persistent state into the inputs civitas-core
//! consumes. They are read-only; no audit log writes here.

use sqlx::PgExecutor;

use civitas_core::{
    eligibility::{EligibilityPolicy, UserVerificationStatus},
    EligibleUser,
};
use civitas_types::{UserId, Weight};

use crate::DbResult;

/// Load the universe of eligible users for a proposal under the given policy.
/// In v1 every eligible user has weight 1.
pub async fn load_eligible_users<'c, E: PgExecutor<'c>>(
    conn: E,
    policy: EligibilityPolicy,
) -> DbResult<Vec<EligibleUser>> {
    let rows = match policy {
        EligibilityPolicy::EmailVerified => sqlx::query!(
            r#"
                select id as "id: UserId"
                from users
                where deleted_at is null and email_verified_at is not null
                "#,
        )
        .fetch_all(conn)
        .await?
        .into_iter()
        .map(|r| EligibleUser {
            user_id: r.id,
            weight: Weight::ONE,
        })
        .collect(),
        EligibilityPolicy::EmailAndPhoneVerified => sqlx::query!(
            r#"
            select id as "id: UserId"
            from users
            where deleted_at is null
              and email_verified_at is not null
              and phone_verified_at is not null
            "#,
        )
        .fetch_all(conn)
        .await?
        .into_iter()
        .map(|r| EligibleUser {
            user_id: r.id,
            weight: Weight::ONE,
        })
        .collect(),
    };

    Ok(rows)
}

/// Verification status for a single user, suitable for [`civitas_core::is_eligible`].
pub async fn load_verification_status<'c, E: PgExecutor<'c>>(
    conn: E,
    user_id: UserId,
) -> DbResult<Option<UserVerificationStatus>> {
    let row = sqlx::query!(
        r#"
        select email_verified_at, phone_verified_at, deleted_at
        from users
        where id = $1
        "#,
        user_id.into_inner(),
    )
    .fetch_optional(conn)
    .await?;

    Ok(row.map(|r| UserVerificationStatus {
        user_id,
        email_verified_at: r.email_verified_at,
        phone_verified_at: r.phone_verified_at,
        deleted_at: r.deleted_at,
    }))
}

//! Users.
//!
//! The `User` struct never carries `password_hash` — that field is loaded
//! only by the auth-specific lookups and only ever passed to verification
//! routines. Callers that want to display or log a user use [`User`];
//! callers that need to authenticate use [`find_password_hash_by_email`].

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgExecutor, Postgres, Transaction};

use civitas_types::UserId;

use crate::audit::{write_log, Action};
use crate::{DbError, DbResult};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: UserId,
    pub email: String,
    pub email_verified_at: Option<DateTime<Utc>>,
    pub phone: Option<String>,
    pub phone_verified_at: Option<DateTime<Utc>>,
    pub display_name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl User {
    #[must_use]
    pub fn is_email_verified(&self) -> bool {
        self.email_verified_at.is_some()
    }

    #[must_use]
    pub fn is_active(&self) -> bool {
        self.deleted_at.is_none()
    }
}

/// Input for [`create`].
#[derive(Debug, Clone)]
pub struct NewUser<'a> {
    pub email: &'a str,
    pub password_hash: &'a str,
    pub display_name: &'a str,
}

/// Insert a new user and write the `user.registered` audit row in the same
/// transaction. Returns the inserted [`User`] (without the password hash).
pub async fn create(tx: &mut Transaction<'_, Postgres>, new: NewUser<'_>) -> DbResult<User> {
    let id = UserId::new();

    let user = sqlx::query_as!(
        User,
        r#"
        insert into users (id, email, password_hash, display_name)
        values ($1, $2, $3, $4)
        returning
            id as "id: UserId",
            email,
            email_verified_at,
            phone,
            phone_verified_at,
            display_name,
            created_at,
            updated_at,
            deleted_at
        "#,
        id.into_inner(),
        new.email,
        new.password_hash,
        new.display_name,
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(map_unique_violation_to_email_taken)?;

    write_log(
        &mut **tx,
        Some(user.id),
        Action::UserRegistered,
        "user",
        user.id.into_inner(),
        None,
    )
    .await?;

    Ok(user)
}

pub async fn find_by_id<'c, E: PgExecutor<'c>>(conn: E, id: UserId) -> DbResult<Option<User>> {
    let row = sqlx::query_as!(
        User,
        r#"
        select
            id as "id: UserId",
            email,
            email_verified_at,
            phone,
            phone_verified_at,
            display_name,
            created_at,
            updated_at,
            deleted_at
        from users
        where id = $1
        "#,
        id.into_inner(),
    )
    .fetch_optional(conn)
    .await?;
    Ok(row)
}

pub async fn find_by_email<'c, E: PgExecutor<'c>>(conn: E, email: &str) -> DbResult<Option<User>> {
    let row = sqlx::query_as!(
        User,
        r#"
        select
            id as "id: UserId",
            email,
            email_verified_at,
            phone,
            phone_verified_at,
            display_name,
            created_at,
            updated_at,
            deleted_at
        from users
        where email = $1::citext
        "#,
        email,
    )
    .fetch_optional(conn)
    .await?;
    Ok(row)
}

/// Used by the login flow only. Returns `None` if the user does not exist or
/// is soft-deleted (callers should not be able to distinguish the two).
pub async fn find_password_hash_by_email<'c, E: PgExecutor<'c>>(
    conn: E,
    email: &str,
) -> DbResult<Option<(UserId, String)>> {
    let row = sqlx::query!(
        r#"
        select id as "id: UserId", password_hash
        from users
        where email = $1::citext and deleted_at is null
        "#,
        email,
    )
    .fetch_optional(conn)
    .await?;
    Ok(row.map(|r| (r.id, r.password_hash)))
}

/// Used by password-change and reset flows.
pub async fn find_password_hash_by_id<'c, E: PgExecutor<'c>>(
    conn: E,
    id: UserId,
) -> DbResult<Option<String>> {
    let row = sqlx::query_scalar!(
        r#"select password_hash from users where id = $1 and deleted_at is null"#,
        id.into_inner(),
    )
    .fetch_optional(conn)
    .await?;
    Ok(row)
}

pub async fn update_password_hash(
    tx: &mut Transaction<'_, Postgres>,
    id: UserId,
    new_hash: &str,
) -> DbResult<()> {
    let updated = sqlx::query!(
        r#"update users set password_hash = $1 where id = $2 and deleted_at is null"#,
        new_hash,
        id.into_inner(),
    )
    .execute(&mut **tx)
    .await?;

    if updated.rows_affected() == 0 {
        return Err(DbError::NotFound);
    }

    write_log(
        &mut **tx,
        Some(id),
        Action::UserPasswordChanged,
        "user",
        id.into_inner(),
        None,
    )
    .await?;

    Ok(())
}

pub async fn mark_email_verified(tx: &mut Transaction<'_, Postgres>, id: UserId) -> DbResult<()> {
    let updated = sqlx::query!(
        r#"
        update users
        set email_verified_at = now()
        where id = $1 and deleted_at is null and email_verified_at is null
        "#,
        id.into_inner(),
    )
    .execute(&mut **tx)
    .await?;

    if updated.rows_affected() > 0 {
        write_log(
            &mut **tx,
            Some(id),
            Action::UserEmailVerified,
            "user",
            id.into_inner(),
            None,
        )
        .await?;
    }
    // Already verified is not an error — idempotent endpoint behavior.

    Ok(())
}

pub async fn mark_phone_verified(
    tx: &mut Transaction<'_, Postgres>,
    id: UserId,
    phone: &str,
) -> DbResult<()> {
    let updated = sqlx::query!(
        r#"
        update users
        set phone = $2, phone_verified_at = now()
        where id = $1 and deleted_at is null
        "#,
        id.into_inner(),
        phone,
    )
    .execute(&mut **tx)
    .await?;

    if updated.rows_affected() == 0 {
        return Err(DbError::NotFound);
    }

    write_log(
        &mut **tx,
        Some(id),
        Action::UserPhoneVerified,
        "user",
        id.into_inner(),
        None,
    )
    .await?;
    Ok(())
}

pub async fn soft_delete(tx: &mut Transaction<'_, Postgres>, id: UserId) -> DbResult<()> {
    let updated = sqlx::query!(
        r#"update users set deleted_at = now() where id = $1 and deleted_at is null"#,
        id.into_inner(),
    )
    .execute(&mut **tx)
    .await?;

    if updated.rows_affected() == 0 {
        return Err(DbError::NotFound);
    }

    write_log(
        &mut **tx,
        Some(id),
        Action::UserDeleted,
        "user",
        id.into_inner(),
        None,
    )
    .await?;

    Ok(())
}

fn map_unique_violation_to_email_taken(err: sqlx::Error) -> DbError {
    if let sqlx::Error::Database(db_err) = &err {
        if db_err.code().as_deref() == Some("23505")
            && db_err.constraint() == Some("users_email_uniq")
        {
            return DbError::EmailAlreadyTaken;
        }
        if db_err.code().as_deref() == Some("23505")
            && db_err.constraint() == Some("users_phone_uniq")
        {
            return DbError::PhoneAlreadyTaken;
        }
    }
    DbError::from(err)
}

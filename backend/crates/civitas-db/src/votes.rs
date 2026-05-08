//! Votes — append-only.
//!
//! Vote rows are inserted, never updated. To "change" a vote during the
//! voting window, application code calls [`record`] again with the new
//! choice; the most-recent row per `(proposal_id, voter_id)` wins at tally
//! time. Old rows are retained for audit.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgExecutor, Postgres, Transaction};

use civitas_core::VoteRecord;
use civitas_types::{ProposalId, ProposalStatus, UserId, VoteChoice, VoteId};

use crate::audit::{write_log, Action};
use crate::{DbError, DbResult};

/// Storage shape for a single vote-cast event. Includes audit fields not
/// surfaced in [`civitas_core::VoteRecord`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VoteRow {
    pub id: VoteId,
    pub proposal_id: ProposalId,
    pub voter_id: UserId,
    pub choice: VoteChoice,
    pub cast_at: DateTime<Utc>,
}

/// Insert a new vote row and write the corresponding audit entry. Rejects
/// proposals that are not in `voting` status — the API layer is supposed to
/// have caught that already, but we defend in depth here so a buggy client
/// can never bypass it.
pub async fn record(
    tx: &mut Transaction<'_, Postgres>,
    proposal_id: ProposalId,
    voter_id: UserId,
    choice: VoteChoice,
) -> DbResult<VoteRow> {
    // Defensive check: the proposal must be in `voting` status and within
    // its declared window.
    let row = sqlx::query!(
        r#"
        select status as "status: ProposalStatus", voting_starts_at, voting_ends_at
        from proposals
        where id = $1
        "#,
        proposal_id.into_inner(),
    )
    .fetch_optional(&mut **tx)
    .await?
    .ok_or(DbError::NotFound)?;

    if row.status != ProposalStatus::Voting {
        return Err(DbError::ProposalNotInVoting);
    }
    let now = Utc::now();
    match (row.voting_starts_at, row.voting_ends_at) {
        (Some(start), Some(end)) if now >= start && now < end => {}
        _ => return Err(DbError::OutsideVotingWindow),
    }

    let id = VoteId::new();
    let vote = sqlx::query_as!(
        VoteRow,
        r#"
        insert into votes (id, proposal_id, voter_id, choice)
        values ($1, $2, $3, $4)
        returning
            id          as "id: VoteId",
            proposal_id as "proposal_id: ProposalId",
            voter_id    as "voter_id: UserId",
            choice      as "choice: VoteChoice",
            cast_at
        "#,
        id.into_inner(),
        proposal_id.into_inner(),
        voter_id.into_inner(),
        choice as VoteChoice,
    )
    .fetch_one(&mut **tx)
    .await?;

    let metadata = serde_json::json!({
        "proposal_id": proposal_id.to_string(),
        "choice": choice.as_str(),
    });
    write_log(
        &mut **tx,
        Some(voter_id),
        Action::VoteCast,
        "vote",
        vote.id.into_inner(),
        Some(&metadata),
    )
    .await?;

    Ok(vote)
}

/// Active votes for a proposal — the most-recent row per voter.
/// Returns the projection consumed by [`civitas_core::tally`].
pub async fn load_active_for_proposal<'c, E: PgExecutor<'c>>(
    conn: E,
    proposal_id: ProposalId,
) -> DbResult<Vec<VoteRecord>> {
    let rows = sqlx::query!(
        r#"
        select distinct on (voter_id)
            id          as "id: VoteId",
            proposal_id as "proposal_id: ProposalId",
            voter_id    as "voter_id: UserId",
            choice      as "choice: VoteChoice",
            cast_at
        from votes
        where proposal_id = $1
        order by voter_id, cast_at desc
        "#,
        proposal_id.into_inner(),
    )
    .fetch_all(conn)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| VoteRecord {
            id: r.id,
            proposal_id: r.proposal_id,
            voter_id: r.voter_id,
            choice: r.choice,
            cast_at: r.cast_at,
        })
        .collect())
}

/// All votes by `voter_id` on `proposal_id`, newest first. Includes
/// superseded rows — votes are append-only, so this is the user's full
/// vote-change history for the proposal.
pub async fn list_history_for_user<'c, E: PgExecutor<'c>>(
    conn: E,
    proposal_id: ProposalId,
    voter_id: UserId,
) -> DbResult<Vec<VoteRow>> {
    let rows = sqlx::query!(
        r#"
        select
            id          as "id: VoteId",
            proposal_id as "proposal_id: ProposalId",
            voter_id    as "voter_id: UserId",
            choice      as "choice: VoteChoice",
            cast_at
        from votes
        where proposal_id = $1 and voter_id = $2
        order by cast_at desc
        "#,
        proposal_id.into_inner(),
        voter_id.into_inner(),
    )
    .fetch_all(conn)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| VoteRow {
            id: r.id,
            proposal_id: r.proposal_id,
            voter_id: r.voter_id,
            choice: r.choice,
            cast_at: r.cast_at,
        })
        .collect())
}

/// The most-recent vote (if any) cast by `voter_id` on `proposal_id`.
pub async fn find_active_for_user<'c, E: PgExecutor<'c>>(
    conn: E,
    proposal_id: ProposalId,
    voter_id: UserId,
) -> DbResult<Option<VoteRow>> {
    let row = sqlx::query!(
        r#"
        select
            id          as "id: VoteId",
            proposal_id as "proposal_id: ProposalId",
            voter_id    as "voter_id: UserId",
            choice      as "choice: VoteChoice",
            cast_at
        from votes
        where proposal_id = $1 and voter_id = $2
        order by cast_at desc
        limit 1
        "#,
        proposal_id.into_inner(),
        voter_id.into_inner(),
    )
    .fetch_optional(conn)
    .await?;

    Ok(row.map(|r| VoteRow {
        id: r.id,
        proposal_id: r.proposal_id,
        voter_id: r.voter_id,
        choice: r.choice,
        cast_at: r.cast_at,
    }))
}

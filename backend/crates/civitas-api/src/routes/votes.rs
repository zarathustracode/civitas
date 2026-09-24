//! Vote-cast and tally endpoints — mounted under `/proposals/:id/...` by
//! [`super::proposals::router`].

use std::collections::HashMap;
use std::convert::Infallible;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::Json;
use futures_util::stream::{self, Stream};

use civitas_core::{eligibility::EligibilityPolicy, tally as core_tally, Tally, TrailKind};
use civitas_db::{delegations, eligibility, proposals, users, votes};
use civitas_types::{ProposalId, TopicId, UserId};

use crate::auth_extractor::{AuthSession, OptionalAuth};
use crate::dto::{CastVoteRequest, NamedUser, TallyResponse, UserTrail, VoteResponse};
use crate::error::{ApiError, ApiResult};
use crate::state::AppState;

pub async fn cast(
    State(state): State<AppState>,
    auth: AuthSession,
    Path(proposal_id): Path<ProposalId>,
    Json(body): Json<CastVoteRequest>,
) -> ApiResult<(StatusCode, Json<VoteResponse>)> {
    if !auth.user.is_email_verified() {
        return Err(ApiError::NotVerified);
    }

    let mut tx = state.pool().begin().await.map_err(ApiError::from)?;
    let row = votes::record(&mut tx, proposal_id, auth.user.id, body.choice)
        .await
        .map_err(ApiError::from)?;
    tx.commit().await.map_err(ApiError::from)?;

    Ok((StatusCode::CREATED, Json(row.into())))
}

/// The requesting user's full vote-change history on this proposal,
/// newest first. Index 0 (if any) is the active vote.
pub async fn list_mine(
    State(state): State<AppState>,
    auth: AuthSession,
    Path(proposal_id): Path<ProposalId>,
) -> ApiResult<Json<Vec<VoteResponse>>> {
    let rows = votes::list_history_for_user(state.pool(), proposal_id, auth.user.id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(rows.into_iter().map(VoteResponse::from).collect()))
}

pub async fn tally_handler(
    State(state): State<AppState>,
    OptionalAuth(auth): OptionalAuth,
    Path(proposal_id): Path<ProposalId>,
) -> ApiResult<Json<TallyResponse>> {
    let proposal = proposals::find_by_id(state.pool(), proposal_id)
        .await
        .map_err(ApiError::from)?
        .ok_or(ApiError::NotFound)?;

    let computed = compute_tally(state.pool(), proposal_id, proposal.topic_id).await?;

    let your_trail = if let Some(session) = auth {
        resolve_user_trail(state.pool(), session.user.id, &computed.tally.trail).await?
    } else {
        None
    };

    Ok(Json(TallyResponse {
        proposal_id,
        yes: computed.tally.yes,
        no: computed.tally.no,
        abstain: computed.tally.abstain,
        eligible_voters: computed.eligible_voters,
        counted_voters: computed.counted_voters,
        your_trail,
    }))
}

/// Server-sent events carrying the proposal's public tally: one `tally`
/// event as soon as it is known, then one per change. The per-viewer trail
/// is not streamed — it only changes through the viewer's own actions or
/// their delegates', and the page reloads it on those.
pub async fn tally_stream(
    State(state): State<AppState>,
    Path(proposal_id): Path<ProposalId>,
) -> ApiResult<Sse<impl Stream<Item = Result<Event, Infallible>>>> {
    let proposal = proposals::find_by_id(state.pool(), proposal_id)
        .await
        .map_err(ApiError::from)?
        .ok_or(ApiError::NotFound)?;

    let mut rx = state.tally_hub().subscribe(proposal_id, proposal.topic_id);
    // Deliver the current value (if already computed) before any change.
    rx.mark_changed();

    let events = stream::unfold(rx, |mut rx| async move {
        loop {
            // Err: the hub dropped this proposal, which ends the stream.
            rx.changed().await.ok()?;
            let Some(update) = rx.borrow_and_update().clone() else {
                continue;
            };
            match serde_json::to_string(&update) {
                Ok(json) => return Some((Ok(Event::default().event("tally").data(json)), rx)),
                Err(error) => tracing::warn!(?error, "serializing tally update failed"),
            }
        }
    });

    Ok(Sse::new(events).keep_alive(KeepAlive::default()))
}

/// A proposal's tally against the current eligible set, with the voter
/// counts reported alongside it.
pub(crate) struct ComputedTally {
    pub tally: Tally,
    pub eligible_voters: usize,
    /// Eligible users whose weight reached a vote, directly or delegated.
    pub counted_voters: usize,
}

pub(crate) async fn compute_tally(
    pool: &sqlx::PgPool,
    proposal_id: ProposalId,
    topic_id: TopicId,
) -> ApiResult<ComputedTally> {
    let active_votes = votes::load_active_for_proposal(pool, proposal_id)
        .await
        .map_err(ApiError::from)?;
    let active_dels = delegations::load_active_for_topic(pool, topic_id)
        .await
        .map_err(ApiError::from)?;
    let eligible = eligibility::load_eligible_users(pool, EligibilityPolicy::EmailVerified)
        .await
        .map_err(ApiError::from)?;

    let tally = core_tally(
        proposal_id,
        topic_id,
        &active_votes,
        &active_dels,
        &eligible,
    );
    let counted_voters = tally
        .trail
        .iter()
        .filter(|t| {
            matches!(
                t.kind,
                TrailKind::Direct { .. } | TrailKind::Delegated { .. }
            )
        })
        .count();

    Ok(ComputedTally {
        tally,
        eligible_voters: eligible.len(),
        counted_voters,
    })
}

/// Find the requesting user's trail entry and resolve any UUIDs in the
/// delegation chain to display names. Returns `None` if the user is not in
/// the trail (i.e. not eligible for this proposal).
async fn resolve_user_trail(
    pool: &sqlx::PgPool,
    user_id: UserId,
    trail: &[civitas_core::TrailEntry],
) -> ApiResult<Option<UserTrail>> {
    let Some(entry) = trail.iter().find(|t| t.user_id == user_id) else {
        return Ok(None);
    };

    let resolved = match &entry.kind {
        civitas_core::TrailKind::Direct { choice } => UserTrail::Direct { choice: *choice },
        civitas_core::TrailKind::Delegated {
            path,
            terminal,
            choice,
        } => {
            let mut needed: Vec<UserId> = path.clone();
            needed.push(*terminal);
            let names: HashMap<UserId, String> = users::list_display_info_by_ids(pool, &needed)
                .await
                .map_err(ApiError::from)?
                .into_iter()
                .collect();
            let to_named = |id: UserId| NamedUser {
                id,
                display_name: names.get(&id).cloned().unwrap_or_else(|| id.to_string()),
            };
            UserTrail::Delegated {
                path: path.iter().copied().map(to_named).collect(),
                terminal: to_named(*terminal),
                choice: *choice,
            }
        }
        civitas_core::TrailKind::NotCounted { reason } => UserTrail::NotCounted {
            reason: match reason {
                civitas_core::NotCountedReason::NoDirectVoteInChain => {
                    "no_direct_vote_in_chain".to_string()
                }
                civitas_core::NotCountedReason::DepthExceeded => "depth_exceeded".to_string(),
            },
        },
    };
    Ok(Some(resolved))
}

//! Vote-cast and tally endpoints — mounted under `/proposals/:id/...` by
//! [`super::proposals::router`].

use std::collections::HashMap;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;

use civitas_core::{eligibility::EligibilityPolicy, tally as core_tally};
use civitas_db::{delegations, eligibility, proposals, users, votes};
use civitas_types::{ProposalId, UserId};

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

pub async fn tally_handler(
    State(state): State<AppState>,
    OptionalAuth(auth): OptionalAuth,
    Path(proposal_id): Path<ProposalId>,
) -> ApiResult<Json<TallyResponse>> {
    let proposal = proposals::find_by_id(state.pool(), proposal_id)
        .await
        .map_err(ApiError::from)?
        .ok_or(ApiError::NotFound)?;

    let active_votes = votes::load_active_for_proposal(state.pool(), proposal_id)
        .await
        .map_err(ApiError::from)?;
    let active_dels = delegations::load_active_for_topic(state.pool(), proposal.topic_id)
        .await
        .map_err(ApiError::from)?;
    let eligible = eligibility::load_eligible_users(state.pool(), EligibilityPolicy::EmailVerified)
        .await
        .map_err(ApiError::from)?;

    let result = core_tally(
        proposal_id,
        proposal.topic_id,
        &active_votes,
        &active_dels,
        &eligible,
    );

    let counted_voters = result
        .trail
        .iter()
        .filter(|t| {
            matches!(
                t.kind,
                civitas_core::TrailKind::Direct { .. } | civitas_core::TrailKind::Delegated { .. }
            )
        })
        .count();

    let your_trail = if let Some(session) = auth {
        resolve_user_trail(state.pool(), session.user.id, &result.trail).await?
    } else {
        None
    };

    Ok(Json(TallyResponse {
        proposal_id,
        yes: result.yes,
        no: result.no,
        abstain: result.abstain,
        eligible_voters: eligible.len(),
        counted_voters,
        your_trail,
    }))
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
            let names: HashMap<UserId, String> =
                users::list_display_info_by_ids(pool, &needed)
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

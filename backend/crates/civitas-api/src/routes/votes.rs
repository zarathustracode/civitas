//! Vote-cast and tally endpoints — mounted under `/proposals/:id/...` by
//! [`super::proposals::router`].

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;

use civitas_core::{eligibility::EligibilityPolicy, tally as core_tally};
use civitas_db::{delegations, eligibility, proposals, votes};
use civitas_types::ProposalId;

use crate::auth_extractor::AuthSession;
use crate::dto::{CastVoteRequest, TallyResponse, VoteResponse};
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

    Ok(Json(TallyResponse {
        proposal_id,
        yes: result.yes,
        no: result.no,
        abstain: result.abstain,
        eligible_voters: eligible.len(),
        counted_voters,
    }))
}

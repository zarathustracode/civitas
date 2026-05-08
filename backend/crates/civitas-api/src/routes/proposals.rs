//! Proposal routes — including the per-proposal vote and tally endpoints,
//! delegated to [`super::votes`] for the actual logic.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;

use civitas_types::{ProposalId, ProposalStatus, TopicId};

use civitas_db::proposals;

use crate::auth_extractor::AuthSession;
use crate::dto::{CreateProposalRequest, ProposalResponse, TransitionStatusRequest};
use crate::error::{ApiError, ApiResult};
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list).post(create))
        .route("/:id", get(by_id))
        .route("/:id/status", post(transition_status))
        .route("/:id/votes", post(super::votes::cast))
        .route("/:id/votes/mine", get(super::votes::list_mine))
        .route("/:id/tally", get(super::votes::tally_handler))
        .route(
            "/:id/comments",
            get(super::comments::list).post(super::comments::create),
        )
}

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    pub topic_id: Option<TopicId>,
    pub status: Option<ProposalStatus>,
}

async fn list(
    State(state): State<AppState>,
    Query(q): Query<ListQuery>,
) -> ApiResult<Json<Vec<ProposalResponse>>> {
    let rows = match (q.topic_id, q.status) {
        (Some(t), s) => proposals::list_by_topic(state.pool(), t, s)
            .await
            .map_err(ApiError::from)?,
        (None, Some(s)) => proposals::list_by_status(state.pool(), s)
            .await
            .map_err(ApiError::from)?,
        (None, None) => proposals::list_by_status(state.pool(), ProposalStatus::Voting)
            .await
            .map_err(ApiError::from)?,
    };
    Ok(Json(rows.into_iter().map(ProposalResponse::from).collect()))
}

async fn by_id(
    State(state): State<AppState>,
    Path(id): Path<ProposalId>,
) -> ApiResult<Json<ProposalResponse>> {
    let p = proposals::find_by_id(state.pool(), id)
        .await
        .map_err(ApiError::from)?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(p.into()))
}

async fn create(
    State(state): State<AppState>,
    auth: AuthSession,
    Json(body): Json<CreateProposalRequest>,
) -> ApiResult<(StatusCode, Json<ProposalResponse>)> {
    if !auth.user.is_email_verified() {
        return Err(ApiError::NotVerified);
    }

    let mut tx = state.pool().begin().await.map_err(ApiError::from)?;
    let proposal = proposals::create(
        &mut tx,
        proposals::NewProposal {
            topic_id: body.topic_id,
            author_id: auth.user.id,
            title: &body.title,
            summary: &body.summary,
            body: &body.body,
        },
    )
    .await
    .map_err(ApiError::from)?;
    tx.commit().await.map_err(ApiError::from)?;

    Ok((StatusCode::CREATED, Json(proposal.into())))
}

async fn transition_status(
    State(state): State<AppState>,
    auth: AuthSession,
    Path(id): Path<ProposalId>,
    Json(body): Json<TransitionStatusRequest>,
) -> ApiResult<Json<ProposalResponse>> {
    let proposal = proposals::find_by_id(state.pool(), id)
        .await
        .map_err(ApiError::from)?
        .ok_or(ApiError::NotFound)?;
    if proposal.author_id != auth.user.id {
        return Err(ApiError::Forbidden);
    }

    let voting_window = match (body.voting_starts_at, body.voting_ends_at) {
        (Some(a), Some(b)) => Some((a, b)),
        (None, None) => None,
        _ => return Err(ApiError::VotingWindowInvalid),
    };

    let mut tx = state.pool().begin().await.map_err(ApiError::from)?;
    proposals::transition_status(&mut tx, auth.user.id, id, body.target, voting_window)
        .await
        .map_err(ApiError::from)?;
    tx.commit().await.map_err(ApiError::from)?;

    let updated = proposals::find_by_id(state.pool(), id)
        .await
        .map_err(ApiError::from)?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(updated.into()))
}

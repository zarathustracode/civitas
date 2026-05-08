//! Topic routes.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};

use civitas_db::{delegations, proposals, topics};
use civitas_types::ProposalStatus;

use crate::auth_extractor::AuthSession;
use crate::dto::{
    CreateTopicRequest, ProposalCounts, TopDelegate, TopicResponse, TopicStatsResponse,
};
use crate::error::{ApiError, ApiResult};
use crate::state::AppState;

const TOP_DELEGATES_LIMIT: i64 = 5;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list).post(create))
        .route("/:slug", get(by_slug))
        .route("/:slug/stats", get(stats))
}

async fn list(State(state): State<AppState>) -> ApiResult<Json<Vec<TopicResponse>>> {
    let topics = topics::list(state.pool()).await.map_err(ApiError::from)?;
    Ok(Json(topics.into_iter().map(TopicResponse::from).collect()))
}

async fn by_slug(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> ApiResult<Json<TopicResponse>> {
    let topic = topics::find_by_slug(state.pool(), &slug)
        .await
        .map_err(ApiError::from)?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(topic.into()))
}

async fn stats(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> ApiResult<Json<TopicStatsResponse>> {
    let topic = topics::find_by_slug(state.pool(), &slug)
        .await
        .map_err(ApiError::from)?
        .ok_or(ApiError::NotFound)?;

    let counts_raw = proposals::count_by_topic_status(state.pool(), topic.id)
        .await
        .map_err(ApiError::from)?;
    let mut counts = ProposalCounts::default();
    for (status, n) in counts_raw {
        match status {
            ProposalStatus::Draft => counts.draft = n,
            ProposalStatus::Deliberation => counts.deliberation = n,
            ProposalStatus::Voting => counts.voting = n,
            ProposalStatus::Closed => counts.closed = n,
        }
    }

    let active_delegations = delegations::count_active_by_topic(state.pool(), topic.id)
        .await
        .map_err(ApiError::from)?;
    let top = delegations::top_delegates_by_topic(state.pool(), topic.id, TOP_DELEGATES_LIMIT)
        .await
        .map_err(ApiError::from)?;
    let top_delegates: Vec<TopDelegate> = top
        .into_iter()
        .map(|(id, display_name, incoming)| TopDelegate {
            id,
            display_name,
            incoming,
        })
        .collect();

    Ok(Json(TopicStatsResponse {
        topic_id: topic.id,
        proposal_counts: counts,
        active_delegations,
        top_delegates,
    }))
}

async fn create(
    State(state): State<AppState>,
    auth: AuthSession,
    Json(body): Json<CreateTopicRequest>,
) -> ApiResult<(StatusCode, Json<TopicResponse>)> {
    if !auth.user.is_email_verified() {
        return Err(ApiError::NotVerified);
    }

    let mut tx = state.pool().begin().await.map_err(ApiError::from)?;
    let topic = topics::create(
        &mut tx,
        auth.user.id,
        topics::NewTopic {
            slug: &body.slug,
            name: &body.name,
            description: &body.description,
        },
    )
    .await
    .map_err(ApiError::from)?;
    tx.commit().await.map_err(ApiError::from)?;

    Ok((StatusCode::CREATED, Json(topic.into())))
}

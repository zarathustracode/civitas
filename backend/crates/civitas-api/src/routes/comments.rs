//! Deliberation comment routes — list/post under `/proposals/:id/comments`,
//! delete under `/comments/:id`.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::delete;
use axum::{Json, Router};

use civitas_db::comments;
use civitas_types::{CommentId, ProposalId};

use crate::auth_extractor::AuthSession;
use crate::dto::{CommentResponse, CreateCommentRequest};
use crate::error::{ApiError, ApiResult};
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/:id", delete(delete_by_author))
}

pub async fn list(
    State(state): State<AppState>,
    Path(proposal_id): Path<ProposalId>,
) -> ApiResult<Json<Vec<CommentResponse>>> {
    let rows = comments::list_thread(state.pool(), proposal_id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(rows.into_iter().map(CommentResponse::from).collect()))
}

pub async fn create(
    State(state): State<AppState>,
    auth: AuthSession,
    Path(proposal_id): Path<ProposalId>,
    Json(body): Json<CreateCommentRequest>,
) -> ApiResult<(StatusCode, Json<CommentResponse>)> {
    if !auth.user.is_email_verified() {
        return Err(ApiError::NotVerified);
    }

    let mut tx = state.pool().begin().await.map_err(ApiError::from)?;
    let row = comments::create(
        &mut tx,
        comments::NewComment {
            proposal_id,
            author_id: auth.user.id,
            parent_id: body.parent_id,
            body: &body.body,
            stance: body.stance,
        },
    )
    .await
    .map_err(ApiError::from)?;
    tx.commit().await.map_err(ApiError::from)?;

    Ok((StatusCode::CREATED, Json(row.into())))
}

async fn delete_by_author(
    State(state): State<AppState>,
    auth: AuthSession,
    Path(id): Path<CommentId>,
) -> ApiResult<StatusCode> {
    let mut tx = state.pool().begin().await.map_err(ApiError::from)?;
    comments::delete_by_author(&mut tx, auth.user.id, id)
        .await
        .map_err(ApiError::from)?;
    tx.commit().await.map_err(ApiError::from)?;
    Ok(StatusCode::NO_CONTENT)
}

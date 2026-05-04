//! Delegation routes.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};

use civitas_db::delegations;
use civitas_types::DelegationId;

use crate::auth_extractor::AuthSession;
use crate::dto::{CreateDelegationRequest, DelegationResponse};
use crate::error::{ApiError, ApiResult};
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_mine).post(create))
        .route("/:id", axum::routing::delete(revoke))
}

async fn list_mine(
    State(state): State<AppState>,
    auth: AuthSession,
) -> ApiResult<Json<Vec<DelegationResponse>>> {
    let rows = delegations::list_active_by_delegator(state.pool(), auth.user.id)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(
        rows.into_iter().map(DelegationResponse::from).collect(),
    ))
}

async fn create(
    State(state): State<AppState>,
    auth: AuthSession,
    Json(body): Json<CreateDelegationRequest>,
) -> ApiResult<(StatusCode, Json<DelegationResponse>)> {
    if !auth.user.is_email_verified() {
        return Err(ApiError::NotVerified);
    }

    let mut tx = state.pool().begin().await.map_err(ApiError::from)?;
    let row = delegations::create_with_cycle_check(
        &mut tx,
        auth.user.id,
        body.delegate_id,
        body.topic_id,
    )
    .await
    .map_err(ApiError::from)?;
    tx.commit().await.map_err(ApiError::from)?;

    Ok((StatusCode::CREATED, Json(row.into())))
}

async fn revoke(
    State(state): State<AppState>,
    auth: AuthSession,
    Path(id): Path<DelegationId>,
) -> ApiResult<StatusCode> {
    // Authorization: only the delegator may revoke. Look up the row to check.
    let existing = delegations::list_active_by_delegator(state.pool(), auth.user.id)
        .await
        .map_err(ApiError::from)?;
    if !existing.iter().any(|d| d.id == id) {
        return Err(ApiError::Forbidden);
    }

    let mut tx = state.pool().begin().await.map_err(ApiError::from)?;
    delegations::revoke(&mut tx, auth.user.id, id)
        .await
        .map_err(ApiError::from)?;
    tx.commit().await.map_err(ApiError::from)?;
    Ok(StatusCode::NO_CONTENT)
}

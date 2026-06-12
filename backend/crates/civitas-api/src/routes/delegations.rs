//! Delegation routes.

use std::collections::HashMap;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};

use civitas_db::{delegations, users};
use civitas_types::{DelegationId, UserId};

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

    let delegate_ids: Vec<UserId> = rows.iter().map(|r| r.delegate_id).collect();
    let names: HashMap<UserId, String> =
        users::list_display_info_by_ids(state.pool(), &delegate_ids)
            .await
            .map_err(ApiError::from)?
            .into_iter()
            .collect();

    let responses: Vec<DelegationResponse> = rows
        .into_iter()
        .map(|row| {
            let mut resp = DelegationResponse::from(row);
            resp.delegate_display_name = names.get(&resp.delegate_id).cloned();
            resp
        })
        .collect();
    Ok(Json(responses))
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

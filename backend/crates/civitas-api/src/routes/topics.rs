//! Topic routes.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};

use civitas_db::topics;

use crate::auth_extractor::AuthSession;
use crate::dto::{CreateTopicRequest, TopicResponse};
use crate::error::{ApiError, ApiResult};
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list).post(create))
        .route("/:slug", get(by_slug))
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

//! User-directory endpoints. Auth-gated; the directory is not public in v1.

use axum::extract::{Query, State};
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;

use civitas_db::users;

use crate::auth_extractor::AuthSession;
use crate::dto::NamedUser;
use crate::error::{ApiError, ApiResult};
use crate::state::AppState;

const SEARCH_LIMIT: i64 = 20;
const MIN_QUERY_LEN: usize = 2;

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: String,
}

pub fn router() -> Router<AppState> {
    Router::new().route("/search", get(search))
}

async fn search(
    State(state): State<AppState>,
    auth: AuthSession,
    Query(SearchQuery { q }): Query<SearchQuery>,
) -> ApiResult<Json<Vec<NamedUser>>> {
    let trimmed = q.trim();
    if trimmed.len() < MIN_QUERY_LEN {
        return Ok(Json(Vec::new()));
    }
    let rows = users::search_active_for_delegate(state.pool(), trimmed, auth.user.id, SEARCH_LIMIT)
        .await
        .map_err(ApiError::from)?;
    Ok(Json(
        rows.into_iter()
            .map(|(id, display_name)| NamedUser { id, display_name })
            .collect(),
    ))
}

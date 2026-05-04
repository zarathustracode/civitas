//! Axum extractors for authenticated and optional-auth requests.
//!
//! [`AuthSession`] requires a valid session cookie and returns the resolved
//! user. Handlers list it directly as a parameter:
//!
//! ```ignore
//! async fn handler(auth: AuthSession, State(state): State<AppState>) { … }
//! ```
//!
//! [`OptionalAuth`] tolerates anonymous callers — useful for read endpoints
//! that change shape when the caller is logged in.

use async_trait::async_trait;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum_extra::extract::cookie::CookieJar;

use civitas_auth::session;
use civitas_db::{sessions::SessionRow, users::User};

use crate::error::ApiError;
use crate::state::AppState;

#[derive(Debug, Clone)]
pub struct AuthSession {
    pub user: User,
    pub session: SessionRow,
}

pub struct OptionalAuth(pub Option<AuthSession>);

#[async_trait]
impl FromRequestParts<AppState> for AuthSession {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let jar = CookieJar::from_headers(&parts.headers);
        let cookie_name = state.config().cookie.session_name.as_str();
        let cookie_value = jar
            .get(cookie_name)
            .map(|c| c.value().to_string())
            .ok_or(ApiError::Unauthorized)?;

        let pair = session::validate(state.pool(), &cookie_value)
            .await
            .map_err(ApiError::from)?;

        let (session, user) = pair.ok_or(ApiError::Unauthorized)?;
        Ok(AuthSession { user, session })
    }
}

#[async_trait]
impl FromRequestParts<AppState> for OptionalAuth {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        match AuthSession::from_request_parts(parts, state).await {
            Ok(s) => Ok(OptionalAuth(Some(s))),
            Err(ApiError::Unauthorized) => Ok(OptionalAuth(None)),
            Err(e) => Err(e),
        }
    }
}

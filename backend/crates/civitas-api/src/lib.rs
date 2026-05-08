//! Civitas HTTP API.
//!
//! Wires routes and middleware over `civitas-auth`, `civitas-db`, and
//! `civitas-core`. The binary entrypoint is `src/main.rs`.

#![doc(html_root_url = "https://docs.rs/civitas-api/0.1.0")]

pub mod auth_extractor;
pub mod config;
pub mod cookies;
pub mod dto;
pub mod error;
pub mod jobs;
pub mod routes;
pub mod state;

use axum::http::HeaderName;
use axum::{routing::get, Json, Router};
use serde_json::json;
use tower_http::cors::{Any, CorsLayer};
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::trace::TraceLayer;

pub use config::Config;
pub use error::{ApiError, ApiResult};
pub use state::AppState;

const REQUEST_ID_HEADER: HeaderName = HeaderName::from_static("x-request-id");

/// Build the top-level router. Accepts an [`AppState`] so tests can supply
/// a different database pool.
pub fn router(state: AppState) -> Router {
    let api = Router::new()
        .route("/health", get(health))
        .nest("/auth", routes::auth::router())
        .nest("/topics", routes::topics::router())
        .nest("/proposals", routes::proposals::router())
        .nest("/delegations", routes::delegations::router())
        .nest("/comments", routes::comments::router())
        .nest("/users", routes::users::router());

    api.with_state(state)
        .layer(SetRequestIdLayer::new(REQUEST_ID_HEADER, MakeRequestUuid))
        .layer(PropagateRequestIdLayer::new(REQUEST_ID_HEADER))
        .layer(TraceLayer::new_for_http())
        .layer(cors())
}

fn cors() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(Any)
        .allow_headers(Any)
        .allow_methods(Any)
}

async fn health() -> Json<serde_json::Value> {
    Json(json!({
        "status": "ok",
        "service": "civitas-api",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

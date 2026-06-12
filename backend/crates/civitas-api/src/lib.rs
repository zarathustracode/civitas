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
pub mod mailer;
pub mod routes;
pub mod security;
pub mod state;

use std::sync::Arc;

use axum::http::{header, HeaderName, HeaderValue, Method};
use axum::response::{IntoResponse, Response};
use axum::{routing::get, Json, Router};
use serde_json::json;
use tower_governor::governor::GovernorConfigBuilder;
use tower_governor::key_extractor::{KeyExtractor, PeerIpKeyExtractor, SmartIpKeyExtractor};
use tower_governor::{GovernorError, GovernorLayer};
use tower_http::cors::CorsLayer;
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::trace::TraceLayer;

use crate::config::RateLimitConfig;

pub use config::Config;
pub use error::{ApiError, ApiResult};
pub use state::AppState;

const REQUEST_ID_HEADER: HeaderName = HeaderName::from_static("x-request-id");

/// Build the top-level router. Accepts an [`AppState`] so tests can supply
/// a different database pool.
///
/// Must be called within a tokio runtime: the rate limiters spawn pruning
/// tasks.
pub fn router(state: AppState) -> Router {
    let rate = state.config().rate_limit.clone();
    let public_base_url = state.config().public_base_url.clone();

    // /auth gets its own, much stricter bucket: those endpoints are
    // unauthenticated and either expensive (Argon2) or send email.
    let auth_routes = rate_limited(
        routes::auth::router(),
        &rate,
        rate.auth_burst,
        rate.auth_replenish_ms,
    );

    let api = Router::new()
        .route("/health", get(health))
        .nest("/auth", auth_routes)
        .nest("/topics", routes::topics::router())
        .nest("/proposals", routes::proposals::router())
        .nest("/delegations", routes::delegations::router())
        .nest("/comments", routes::comments::router())
        .nest("/users", routes::users::router())
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            security::verify_origin,
        ));

    let api = rate_limited(api, &rate, rate.global_burst, rate.global_replenish_ms);

    api.with_state(state)
        .layer(SetRequestIdLayer::new(REQUEST_ID_HEADER, MakeRequestUuid))
        .layer(PropagateRequestIdLayer::new(REQUEST_ID_HEADER))
        .layer(TraceLayer::new_for_http())
        .layer(cors(&public_base_url))
}

/// Wrap `router` in a per-IP token bucket: `burst` capacity, one token
/// back every `replenish_ms`. The client IP comes from the socket peer
/// address, or from forwarded headers when `TRUST_PROXY` is set (required
/// when all traffic funnels through a reverse proxy or the `SvelteKit`
/// server, which would otherwise share one bucket across every user).
/// Spawns a pruning task so the per-key state map does not grow unbounded.
fn rate_limited(
    router: Router<AppState>,
    rate: &RateLimitConfig,
    burst: u32,
    replenish_ms: u64,
) -> Router<AppState> {
    if rate.trust_proxy {
        apply_governor(router, SmartIpKeyExtractor, burst, replenish_ms)
    } else {
        apply_governor(router, PeerIpKeyExtractor, burst, replenish_ms)
    }
}

fn apply_governor<K>(
    router: Router<AppState>,
    extractor: K,
    burst: u32,
    replenish_ms: u64,
) -> Router<AppState>
where
    K: KeyExtractor + Send + Sync + 'static,
    K::Key: Send + Sync + 'static,
{
    let config = Arc::new(
        GovernorConfigBuilder::default()
            .key_extractor(extractor)
            .per_millisecond(replenish_ms)
            .burst_size(burst)
            .error_handler(governor_error)
            .finish()
            .expect("burst and replenish validated nonzero at config load"),
    );

    let limiter = config.limiter().clone();
    tokio::spawn(async move {
        let mut tick = tokio::time::interval(std::time::Duration::from_secs(60));
        loop {
            tick.tick().await;
            limiter.retain_recent();
        }
    });

    router.layer(GovernorLayer { config })
}

/// Translate limiter outcomes into the API's stable JSON error envelope.
/// (By-value signature dictated by the builder's error-handler callback.)
#[allow(clippy::needless_pass_by_value)]
fn governor_error(err: GovernorError) -> Response {
    match err {
        GovernorError::TooManyRequests { .. } => ApiError::RateLimited.into_response(),
        GovernorError::UnableToExtractKey => ApiError::Internal(anyhow::anyhow!(
            "rate limiter could not determine the client address"
        ))
        .into_response(),
        GovernorError::Other { .. } => {
            ApiError::Internal(anyhow::anyhow!("rate limiter failure")).into_response()
        }
    }
}

/// The API serves the configured frontend only — v1 has no public API —
/// so CORS mirrors `PUBLIC_BASE_URL` rather than `Any`. Same-origin
/// deployments (dev proxy, prod path rewrite) never hit CORS at all; this
/// matters only if a deployment exposes the API on its own origin.
fn cors(public_base_url: &str) -> CorsLayer {
    let Ok(origin) = public_base_url.trim_end_matches('/').parse::<HeaderValue>() else {
        tracing::warn!(
            public_base_url = %public_base_url,
            "PUBLIC_BASE_URL is not a valid CORS origin; cross-origin requests will be denied"
        );
        return CorsLayer::new();
    };
    CorsLayer::new()
        .allow_origin(origin)
        .allow_headers([header::CONTENT_TYPE])
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
        ])
        .allow_credentials(true)
}

async fn health() -> Json<serde_json::Value> {
    Json(json!({
        "status": "ok",
        "service": "civitas-api",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

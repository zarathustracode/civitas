//! Civitas API server.
//!
//! Minimal walking-skeleton entrypoint. Routes for auth, proposals, votes,
//! delegations, and deliberation are added in follow-up sessions; this binary
//! today exposes only `/health` so the wiring is exercised end to end.

use std::net::SocketAddr;

use axum::{routing::get, Json, Router};
use serde_json::json;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();

    let addr: SocketAddr = std::env::var("HTTP_LISTEN_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:8080".to_string())
        .parse()?;

    let app = Router::new().route("/health", get(health));

    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!(%addr, "civitas-api listening");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn health() -> Json<serde_json::Value> {
    Json(json!({
        "status": "ok",
        "service": "civitas-api",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("civitas=info,tower_http=info,sqlx=warn"));
    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer().compact())
        .init();
}

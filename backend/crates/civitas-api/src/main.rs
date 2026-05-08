//! Civitas API server entrypoint.

use std::time::Duration;

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use civitas_api::{jobs, router, AppState, Config};

const DEFAULT_AUTO_CLOSE_INTERVAL_SECS: u64 = 60;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();

    let config = Config::from_env()?;
    tracing::info!(addr = %config.http_listen_addr, "loading config");

    let pool = civitas_db::connect(&config.database_url, config.database_max_connections).await?;
    civitas_db::migrate(&pool).await?;
    tracing::info!("migrations applied");

    let interval = Duration::from_secs(
        std::env::var("AUTO_CLOSE_INTERVAL_SECS")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(DEFAULT_AUTO_CLOSE_INTERVAL_SECS)
            .max(1),
    );
    tokio::spawn(jobs::auto_close_expired_loop(pool.clone(), interval));
    tracing::info!(interval = ?interval, "auto-close job started");

    let addr = config.http_listen_addr;
    let state = AppState::new(pool, config);
    let app = router(state);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!(%addr, "civitas-api listening");
    axum::serve(listener, app).await?;
    Ok(())
}

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("civitas=info,tower_http=info,sqlx=warn"));
    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer().compact())
        .init();
}

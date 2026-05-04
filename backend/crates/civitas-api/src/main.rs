//! Civitas API server entrypoint.

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use civitas_api::{router, AppState, Config};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();

    let config = Config::from_env()?;
    tracing::info!(addr = %config.http_listen_addr, "loading config");

    let pool = civitas_db::connect(&config.database_url, config.database_max_connections).await?;
    civitas_db::migrate(&pool).await?;
    tracing::info!("migrations applied");

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

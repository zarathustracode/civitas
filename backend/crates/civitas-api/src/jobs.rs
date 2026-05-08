//! Background jobs that run alongside the HTTP server.
//!
//! Single-process, in-memory loops. Each job runs as its own tokio task
//! spawned at startup; failures inside a tick are logged and the loop
//! continues — a transient DB error must not stop the job permanently.
//!
//! Operationally these tasks share the API process. If the API restarts,
//! they restart with it. If we ever scale to multiple API instances, jobs
//! that mutate state should be moved behind a row-level lock or a single
//! leader; for v0.1 the deployment target is one instance per database.

use std::time::Duration;

use sqlx::PgPool;
use tokio::time;

use civitas_db::proposals;

/// Run the auto-close-expired-proposals tick on a fixed interval.
///
/// Each tick: open a tx, run [`proposals::auto_close_expired`], commit.
/// Logs the ids closed at info level. Errors are logged at warn and the
/// loop sleeps until the next tick — a flaky DB does not poison the task.
pub async fn auto_close_expired_loop(pool: PgPool, interval: Duration) {
    let mut ticker = time::interval(interval);
    // Don't burst-fire if we miss ticks (e.g. the DB was unreachable).
    ticker.set_missed_tick_behavior(time::MissedTickBehavior::Delay);
    loop {
        ticker.tick().await;
        match run_once(&pool).await {
            Ok(closed) if !closed.is_empty() => {
                tracing::info!(
                    closed_count = closed.len(),
                    closed_ids = ?closed,
                    "auto-closed expired voting proposals"
                );
            }
            Ok(_) => {}
            Err(e) => {
                tracing::warn!(error = %e, "auto-close tick failed; will retry");
            }
        }
    }
}

async fn run_once(pool: &PgPool) -> sqlx::Result<Vec<civitas_types::ProposalId>> {
    let mut tx = pool.begin().await?;
    let ids = proposals::auto_close_expired(&mut tx)
        .await
        .map_err(|e| match e {
            civitas_db::DbError::Sqlx(s) => s,
            other => sqlx::Error::Protocol(other.to_string()),
        })?;
    tx.commit().await?;
    Ok(ids)
}

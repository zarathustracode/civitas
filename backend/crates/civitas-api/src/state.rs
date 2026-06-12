//! Shared application state injected into every handler.
//!
//! Cheap to clone — internally it's `Arc`s. Axum clones it per-request.

use std::sync::Arc;

use sqlx::PgPool;

use civitas_auth::verification::EmailVerificationProvider;

use crate::config::Config;
use crate::mailer::Mailer;

#[derive(Clone)]
pub struct AppState(Arc<AppStateInner>);

pub struct AppStateInner {
    pub pool: PgPool,
    pub config: Config,
    pub email_verification: EmailVerificationProvider,
    pub mailer: Arc<dyn Mailer>,
}

impl AppState {
    #[must_use]
    pub fn new(pool: PgPool, config: Config, mailer: Arc<dyn Mailer>) -> Self {
        Self(Arc::new(AppStateInner {
            pool,
            config,
            email_verification: EmailVerificationProvider::default(),
            mailer,
        }))
    }

    #[must_use]
    pub fn pool(&self) -> &PgPool {
        &self.0.pool
    }

    #[must_use]
    pub fn config(&self) -> &Config {
        &self.0.config
    }

    #[must_use]
    pub fn email_verification(&self) -> &EmailVerificationProvider {
        &self.0.email_verification
    }

    #[must_use]
    pub fn mailer(&self) -> Arc<dyn Mailer> {
        Arc::clone(&self.0.mailer)
    }
}

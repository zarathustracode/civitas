//! Shared harness for the API integration tests: an in-process router over
//! a real Postgres, driven with `tower::ServiceExt::oneshot`.
//!
//! Every test returns early when `DATABASE_URL` is unset so `cargo test`
//! stays green offline; CI sets it. Tests share one database, so fixtures
//! use unique emails and slugs instead of relying on a clean slate.

#![allow(dead_code)] // each test binary uses a different subset

use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use axum::body::Body;
use axum::extract::ConnectInfo;
use axum::http::{header, Request, StatusCode};
use axum::Router;
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

use civitas_api::config::{CookieConfig, MailConfig, RateLimitConfig};
use civitas_api::mailer::{Mail, Mailer, MailerError};
use civitas_api::{router, AppState, Config};

pub const PASSWORD: &str = "correct horse battery staple";
pub const PEER: &str = "192.0.2.10:52000";

/// Collects outbound mail instead of delivering it.
#[derive(Default)]
pub struct CapturingMailer(Mutex<Vec<Mail>>);

#[async_trait]
impl Mailer for CapturingMailer {
    async fn send(&self, mail: Mail) -> Result<(), MailerError> {
        self.0.lock().unwrap().push(mail);
        Ok(())
    }
}

impl CapturingMailer {
    pub fn sent(&self) -> Vec<Mail> {
        self.0.lock().unwrap().clone()
    }
}

pub struct TestApp {
    pub pool: PgPool,
    pub router: Router,
    pub mailer: Arc<CapturingMailer>,
}

pub struct Response {
    pub status: StatusCode,
    pub headers: axum::http::HeaderMap,
    pub body: Value,
}

impl Response {
    /// The `name=value` pair of the first `Set-Cookie`, ready to send back.
    pub fn session_cookie(&self) -> String {
        let raw = self
            .headers
            .get(header::SET_COOKIE)
            .expect("response sets a cookie")
            .to_str()
            .unwrap();
        raw.split(';').next().unwrap().to_string()
    }
}

pub fn unique(prefix: &str) -> String {
    format!("{prefix}-{}", Uuid::now_v7().simple())
}

pub fn test_config(database_url: &str) -> Config {
    Config {
        database_url: database_url.to_string(),
        database_max_connections: 5,
        http_listen_addr: "127.0.0.1:0".parse().unwrap(),
        public_base_url: "http://localhost:5173".to_string(),
        cookie: CookieConfig {
            domain: None,
            secure: false,
            session_name: "civitas_session".to_string(),
        },
        dev_return_verification_token: true,
        mail: MailConfig::Log,
        rate_limit: RateLimitConfig {
            trust_proxy: false,
            auth_burst: 1_000,
            auth_replenish_ms: 1,
            global_burst: 1_000,
            global_replenish_ms: 1,
        },
    }
}

impl TestApp {
    pub async fn new() -> Option<Self> {
        Self::with_config(|_| {}).await
    }

    pub async fn with_config(tweak: impl FnOnce(&mut Config)) -> Option<Self> {
        let url = std::env::var("DATABASE_URL").ok()?;
        let pool = civitas_db::connect(&url, 5).await.ok()?;
        civitas_db::migrate(&pool).await.ok()?;

        let mut config = test_config(&url);
        tweak(&mut config);
        let mailer = Arc::new(CapturingMailer::default());
        let state = AppState::new(pool.clone(), config, mailer.clone());
        Some(Self {
            pool,
            router: router(state),
            mailer,
        })
    }

    /// Send a request as if from the socket peer [`PEER`]. `extra` headers
    /// are applied verbatim.
    pub async fn send(
        &self,
        method: &str,
        path: &str,
        cookie: Option<&str>,
        body: Option<Value>,
        extra: &[(&str, &str)],
    ) -> Response {
        let mut builder = Request::builder().method(method).uri(path);
        if let Some(cookie) = cookie {
            builder = builder.header(header::COOKIE, cookie);
        }
        for (name, value) in extra {
            builder = builder.header(*name, *value);
        }
        let body = match body {
            Some(json) => {
                builder = builder.header(header::CONTENT_TYPE, "application/json");
                Body::from(json.to_string())
            }
            None => Body::empty(),
        };
        let mut request = builder.body(body).unwrap();
        request
            .extensions_mut()
            .insert(ConnectInfo(PEER.parse::<SocketAddr>().unwrap()));

        let response = self.router.clone().oneshot(request).await.unwrap();
        let status = response.status();
        let headers = response.headers().clone();
        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body = if bytes.is_empty() {
            Value::Null
        } else {
            serde_json::from_slice(&bytes).unwrap_or(Value::Null)
        };
        Response {
            status,
            headers,
            body,
        }
    }

    pub async fn get(&self, path: &str, cookie: Option<&str>) -> Response {
        self.send("GET", path, cookie, None, &[]).await
    }

    pub async fn post(&self, path: &str, cookie: Option<&str>, body: Value) -> Response {
        self.send("POST", path, cookie, Some(body), &[]).await
    }

    /// Register and verify a fresh account; returns its email.
    pub async fn verified_user(&self, prefix: &str) -> String {
        let email = format!("{}@example.com", unique(prefix));
        let registered = self
            .post(
                "/auth/register",
                None,
                json!({ "email": email, "password": PASSWORD, "display_name": prefix }),
            )
            .await;
        assert_eq!(
            registered.status,
            StatusCode::CREATED,
            "{:?}",
            registered.body
        );
        let token = registered.body["dev_verification_token"]
            .as_str()
            .expect("dev token echoed")
            .to_string();
        let verified = self
            .post("/auth/verify-email", None, json!({ "token": token }))
            .await;
        assert_eq!(verified.status, StatusCode::OK, "{:?}", verified.body);
        email
    }

    /// Log in with extra request headers; returns the full response.
    pub async fn login_with(&self, email: &str, extra: &[(&str, &str)]) -> Response {
        self.send(
            "POST",
            "/auth/login",
            None,
            Some(json!({ "email": email, "password": PASSWORD })),
            extra,
        )
        .await
    }

    /// Register, verify, and log in; returns `(email, cookie)`.
    pub async fn logged_in_user(&self, prefix: &str) -> (String, String) {
        let email = self.verified_user(prefix).await;
        let login = self.login_with(&email, &[]).await;
        assert_eq!(login.status, StatusCode::OK, "{:?}", login.body);
        (email, login.session_cookie())
    }
}

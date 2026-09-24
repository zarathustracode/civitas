//! Login records the caller's user agent and IP on the session row.

mod common;

use axum::http::StatusCode;

use common::TestApp;

async fn latest_session(app: &TestApp, email: &str) -> (Option<String>, Option<String>) {
    sqlx::query_as(
        "select s.user_agent, s.ip_address
         from sessions s join users u on u.id = s.user_id
         where u.email = $1
         order by s.created_at desc
         limit 1",
    )
    .bind(email)
    .fetch_one(&app.pool)
    .await
    .unwrap()
}

#[tokio::test]
async fn login_records_user_agent_and_peer_ip() {
    let Some(app) = TestApp::new().await else {
        return;
    };
    let email = app.verified_user("ua").await;

    let login = app
        .login_with(
            &email,
            &[
                ("user-agent", "Mozilla/5.0 (civitas-test)"),
                // Ignored: TRUST_PROXY is off, so only the peer counts.
                ("x-forwarded-for", "203.0.113.50"),
            ],
        )
        .await;
    assert_eq!(login.status, StatusCode::OK);

    let (user_agent, ip) = latest_session(&app, &email).await;
    assert_eq!(user_agent.as_deref(), Some("Mozilla/5.0 (civitas-test)"));
    assert_eq!(ip.as_deref(), Some("192.0.2.10"));
}

#[tokio::test]
async fn login_behind_trusted_proxy_records_forwarded_ip() {
    let Some(app) = TestApp::with_config(|c| c.rate_limit.trust_proxy = true).await else {
        return;
    };
    let email = app.verified_user("xff").await;

    let login = app
        .login_with(&email, &[("x-forwarded-for", "203.0.113.50")])
        .await;
    assert_eq!(login.status, StatusCode::OK);

    let (user_agent, ip) = latest_session(&app, &email).await;
    assert_eq!(user_agent, None);
    assert_eq!(ip.as_deref(), Some("203.0.113.50"));
}

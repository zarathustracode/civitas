//! Passwordless sign-in: a link is mailed only to verified accounts, opens
//! exactly one session, and is retired by a newer link or a password reset.

mod common;

use std::time::Duration;

use axum::http::StatusCode;
use serde_json::json;

use common::{token_in, unique_email, TestApp, PASSWORD};

const LINK: &str = "/auth/login-link/confirm";
const RESET: &str = "/auth/reset-password";

async fn request_link(app: &TestApp, email: &str) {
    let r = app
        .post("/auth/login-link/request", None, json!({ "email": email }))
        .await;
    assert_eq!(r.status, StatusCode::ACCEPTED);
}

async fn complete(app: &TestApp, token: &str) -> common::Response {
    app.send(
        "POST",
        "/auth/login-link/complete",
        None,
        Some(json!({ "token": token })),
        &[("user-agent", "civitas-link-test")],
    )
    .await
}

#[tokio::test]
async fn link_signs_in_once() {
    let Some(app) = TestApp::new().await else {
        return;
    };
    let email = app.verified_user("link").await;
    request_link(&app, &email).await;

    let mail = app.mailer.wait_for(&email, LINK, 0).await;
    assert!(mail.html.contains(LINK));
    let token = token_in(&mail);

    let signed_in = complete(&app, &token).await;
    assert_eq!(signed_in.status, StatusCode::OK, "{:?}", signed_in.body);
    assert_eq!(signed_in.body["email"], email);
    let cookie = signed_in.session_cookie();
    let me = app.get("/auth/me", Some(&cookie)).await;
    assert_eq!(me.status, StatusCode::OK);

    let user_agent: Option<String> = sqlx::query_scalar(
        "select s.user_agent from sessions s join users u on u.id = s.user_id
         where u.email = $1 order by s.created_at desc limit 1",
    )
    .bind(&email)
    .fetch_one(&app.pool)
    .await
    .unwrap();
    assert_eq!(user_agent.as_deref(), Some("civitas-link-test"));

    let reused = complete(&app, &token).await;
    assert_eq!(reused.status, StatusCode::BAD_REQUEST);
    assert_eq!(reused.body["error"]["code"], "auth.token_invalid");
}

#[tokio::test]
async fn a_newer_link_retires_the_older_one() {
    let Some(app) = TestApp::new().await else {
        return;
    };
    let email = app.verified_user("relink").await;
    request_link(&app, &email).await;
    let first = token_in(&app.mailer.wait_for(&email, LINK, 0).await);
    request_link(&app, &email).await;
    let second = token_in(&app.mailer.wait_for(&email, LINK, 1).await);

    assert_eq!(complete(&app, &first).await.status, StatusCode::BAD_REQUEST);
    assert_eq!(complete(&app, &second).await.status, StatusCode::OK);
}

#[tokio::test]
async fn password_reset_retires_outstanding_links() {
    let Some(app) = TestApp::new().await else {
        return;
    };
    let email = app.verified_user("reset").await;
    request_link(&app, &email).await;
    let link = token_in(&app.mailer.wait_for(&email, LINK, 0).await);

    let r = app
        .post(
            "/auth/password-reset/request",
            None,
            json!({ "email": email }),
        )
        .await;
    assert_eq!(r.status, StatusCode::ACCEPTED);
    let reset = token_in(&app.mailer.wait_for(&email, RESET, 0).await);
    let done = app
        .post(
            "/auth/password-reset/complete",
            None,
            json!({ "token": reset, "new_password": format!("{PASSWORD} again") }),
        )
        .await;
    assert_eq!(done.status, StatusCode::NO_CONTENT, "{:?}", done.body);

    assert_eq!(complete(&app, &link).await.status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn unknown_and_unverified_addresses_get_the_same_answer_and_no_mail() {
    let Some(app) = TestApp::new().await else {
        return;
    };
    let unknown = unique_email("nobody");
    request_link(&app, &unknown).await;

    let unverified = unique_email("pending");
    let registered = app
        .post(
            "/auth/register",
            None,
            json!({ "email": unverified, "password": PASSWORD, "display_name": "Pending" }),
        )
        .await;
    assert_eq!(registered.status, StatusCode::CREATED);
    request_link(&app, &unverified).await;

    // Registration's own verification mail is the only one either receives.
    tokio::time::sleep(Duration::from_millis(200)).await;
    let sent = app.mailer.sent();
    assert!(sent.iter().all(|m| m.to != unknown));
    assert!(sent
        .iter()
        .filter(|m| m.to == unverified)
        .all(|m| !m.text.contains(LINK)));
}

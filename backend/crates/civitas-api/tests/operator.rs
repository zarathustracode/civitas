//! The operator dashboard is restricted to `OPERATOR_EMAILS`, reports
//! deployment-wide counts, and never exposes how anyone voted.

mod common;

use axum::http::StatusCode;
use serde_json::json;

use common::{unique_email, voting_proposal, TestApp, PASSWORD};

async fn app_with_operator() -> Option<(TestApp, String)> {
    // Registered with capitals; configured lowercase: matching ignores case.
    let email = unique_email("Op");
    let configured = email.to_lowercase();
    let app = TestApp::with_config(|c| c.operator_emails = vec![configured]).await?;
    app.register_verified(&email, "Operator").await;
    Some((app, email))
}

async fn cookie_for(app: &TestApp, email: &str) -> String {
    let login = app
        .post(
            "/auth/login",
            None,
            json!({ "email": email, "password": PASSWORD }),
        )
        .await;
    assert_eq!(login.status, StatusCode::OK);
    login.session_cookie()
}

#[tokio::test]
async fn dashboard_requires_an_operator() {
    let Some((app, _)) = app_with_operator().await else {
        return;
    };
    let anonymous = app.get("/operator/overview", None).await;
    assert_eq!(anonymous.status, StatusCode::UNAUTHORIZED);

    let (_, citizen) = app.logged_in_user("citizen").await;
    let refused = app.get("/operator/overview", Some(&citizen)).await;
    assert_eq!(refused.status, StatusCode::FORBIDDEN);
    let refused = app.get("/operator/audit", Some(&citizen)).await;
    assert_eq!(refused.status, StatusCode::FORBIDDEN);

    let me = app.get("/auth/me", Some(&citizen)).await;
    assert_eq!(me.body["is_operator"], false);
}

#[tokio::test]
async fn overview_counts_the_deployment() {
    let Some((app, email)) = app_with_operator().await else {
        return;
    };
    let cookie = cookie_for(&app, &email).await;
    let proposal_id = voting_proposal(&app, &email).await;

    let me = app.get("/auth/me", Some(&cookie)).await;
    assert_eq!(me.body["is_operator"], true);
    assert_eq!(me.body["email"], email);

    let overview = app.get("/operator/overview", Some(&cookie)).await;
    assert_eq!(overview.status, StatusCode::OK, "{:?}", overview.body);
    let body = overview.body;
    assert!(body["users"]["verified"].as_i64().unwrap() >= 1);
    assert!(body["users"]["registered_last_7_days"].as_i64().unwrap() >= 1);
    assert!(body["proposals"]["voting"].as_i64().unwrap() >= 1);
    assert!(body["active_sessions"].as_i64().unwrap() >= 1);
    let listed = body["voting"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["id"] == proposal_id.to_string())
        .expect("voting proposal listed");
    assert_eq!(listed["counted_voters"], 0);
    // Nothing in the overview carries an email address.
    assert!(!body.to_string().contains('@'));
}

#[tokio::test]
async fn audit_feed_pages_and_hides_ballots() {
    let Some((app, email)) = app_with_operator().await else {
        return;
    };
    let cookie = cookie_for(&app, &email).await;
    let proposal_id = voting_proposal(&app, &email).await;
    let vote = app
        .post(
            &format!("/proposals/{proposal_id}/votes"),
            Some(&cookie),
            json!({ "choice": "no" }),
        )
        .await;
    assert_eq!(vote.status, StatusCode::CREATED);

    let feed = app.get("/operator/audit?limit=200", Some(&cookie)).await;
    assert_eq!(feed.status, StatusCode::OK);
    let entries = feed.body.as_array().unwrap();
    let cast = entries
        .iter()
        .find(|e| {
            e["action"] == "vote.cast" && e["metadata"]["proposal_id"] == proposal_id.to_string()
        })
        .expect("our vote is in the feed");
    assert_eq!(cast["actor_display_name"], "Operator");
    assert_eq!(cast["entity_type"], "vote");
    assert!(cast["metadata"].get("choice").is_none());

    let first = app.get("/operator/audit?limit=1", Some(&cookie)).await;
    let first_id = first.body[0]["id"].as_str().unwrap().to_string();
    let next = app
        .get(
            &format!("/operator/audit?limit=1&before={first_id}"),
            Some(&cookie),
        )
        .await;
    assert_eq!(next.status, StatusCode::OK);
    let next_entry = &next.body[0];
    assert_ne!(next_entry["id"], first.body[0]["id"]);
    assert!(next_entry["created_at"].as_str() <= first.body[0]["created_at"].as_str());
}

//! `GET /proposals/:id/tally/stream` pushes a fresh tally after each vote
//! and when the proposal's status changes.

mod common;

use std::time::Duration;

use axum::body::{Body, BodyDataStream};
use axum::http::{header, Request, StatusCode};
use futures_util::StreamExt;
use serde_json::{json, Value};
use tower::ServiceExt;

use civitas_types::ProposalId;

use common::{voting_proposal, TestApp};

struct EventStream {
    body: BodyDataStream,
    buffer: String,
}

impl EventStream {
    async fn open(app: &TestApp, proposal_id: ProposalId) -> Self {
        let request = Request::builder()
            .uri(format!("/proposals/{proposal_id}/tally/stream"))
            .extension(axum::extract::ConnectInfo(
                common::PEER.parse::<std::net::SocketAddr>().unwrap(),
            ))
            .body(Body::empty())
            .unwrap();
        let response = app.router.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers()[header::CONTENT_TYPE],
            "text/event-stream"
        );
        Self {
            body: response.into_body().into_data_stream(),
            buffer: String::new(),
        }
    }

    /// The next `tally` event's data, skipping keep-alive comments.
    async fn next_tally(&mut self) -> Value {
        loop {
            if let Some(end) = self.buffer.find("\n\n") {
                let frame: String = self.buffer.drain(..end + 2).collect();
                let is_tally = frame.lines().any(|l| l == "event: tally");
                let data = frame.lines().find_map(|l| l.strip_prefix("data: "));
                if let (true, Some(data)) = (is_tally, data) {
                    return serde_json::from_str(data).unwrap();
                }
                continue;
            }
            let chunk = self.body.next().await.expect("stream stays open").unwrap();
            self.buffer.push_str(std::str::from_utf8(&chunk).unwrap());
        }
    }

    /// Wait until an event satisfies `pred`. Other tests share the database,
    /// so unrelated eligibility changes may push extra events first.
    async fn wait_for(&mut self, pred: impl Fn(&Value) -> bool) -> Value {
        tokio::time::timeout(Duration::from_secs(10), async {
            loop {
                let event = self.next_tally().await;
                if pred(&event) {
                    return event;
                }
            }
        })
        .await
        .expect("expected tally event within 10s")
    }
}

#[tokio::test]
async fn stream_pushes_votes_and_status_changes() {
    let Some(app) = TestApp::new().await else {
        return;
    };
    let (email, cookie) = app.logged_in_user("live").await;
    let proposal_id = voting_proposal(&app, &email).await;

    let mut stream = EventStream::open(&app, proposal_id).await;
    let first = stream.wait_for(|_| true).await;
    assert_eq!(first["proposal_id"], proposal_id.to_string());
    assert_eq!(first["status"], "voting");
    assert_eq!(first["yes"], "0");

    let vote = app
        .post(
            &format!("/proposals/{proposal_id}/votes"),
            Some(&cookie),
            json!({ "choice": "yes" }),
        )
        .await;
    assert_eq!(vote.status, StatusCode::CREATED, "{:?}", vote.body);
    let after_vote = stream.wait_for(|e| e["yes"] == "1").await;
    assert_eq!(after_vote["counted_voters"], 1);

    let closed = app
        .post(
            &format!("/proposals/{proposal_id}/status"),
            Some(&cookie),
            json!({ "target": "closed" }),
        )
        .await;
    assert_eq!(closed.status, StatusCode::OK, "{:?}", closed.body);
    let after_close = stream.wait_for(|e| e["status"] == "closed").await;
    assert_eq!(after_close["yes"], "1");
}

#[tokio::test]
async fn second_viewer_gets_the_current_tally_immediately() {
    let Some(app) = TestApp::new().await else {
        return;
    };
    let (email, _cookie) = app.logged_in_user("live2").await;
    let proposal_id = voting_proposal(&app, &email).await;

    let mut first = EventStream::open(&app, proposal_id).await;
    first.wait_for(|_| true).await;

    let mut second = EventStream::open(&app, proposal_id).await;
    let event = second.wait_for(|_| true).await;
    assert_eq!(event["proposal_id"], proposal_id.to_string());
}

#[tokio::test]
async fn unknown_proposal_is_not_found() {
    let Some(app) = TestApp::new().await else {
        return;
    };
    let response = app
        .get(
            &format!("/proposals/{}/tally/stream", ProposalId::new()),
            None,
        )
        .await;
    assert_eq!(response.status, StatusCode::NOT_FOUND);
}

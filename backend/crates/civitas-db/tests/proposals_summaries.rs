//! Integration coverage for the building blocks behind the enriched docket
//! endpoint (`GET /proposals/summaries`): the per-proposal tally and the
//! *visible* comment count (soft-deleted and moderator-hidden comments are
//! excluded). Mirrors the data-layer flow the API handler composes.
//!
//! Skipped silently when `DATABASE_URL` is not set, so `cargo test` stays
//! green offline. CI sets the env explicitly.

use chrono::{Duration, Utc};
use uuid::Uuid;

use civitas_core::{eligibility::EligibilityPolicy, tally};
use civitas_db::{comments, eligibility, proposals, topics, users, votes};
use civitas_types::{ProposalStatus, Stance, VoteChoice, Weight};

fn unique(prefix: &str) -> String {
    format!("{prefix}-{}", Uuid::now_v7().simple())
}

async fn setup_pool() -> Option<sqlx::PgPool> {
    let url = std::env::var("DATABASE_URL").ok()?;
    let pool = civitas_db::connect(&url, 5).await.ok()?;
    civitas_db::migrate(&pool).await.ok()?;
    Some(pool)
}

async fn make_verified_user(pool: &sqlx::PgPool, email_prefix: &str) -> civitas_db::users::User {
    let mut tx = pool.begin().await.unwrap();
    let user = users::create(
        &mut tx,
        users::NewUser {
            email: &format!("{}@example.com", unique(email_prefix)),
            password_hash: "$argon2id$placeholder",
            display_name: email_prefix,
        },
    )
    .await
    .unwrap();
    users::mark_email_verified(&mut tx, user.id).await.unwrap();
    tx.commit().await.unwrap();

    users::find_by_id(pool, user.id).await.unwrap().unwrap()
}

/// A proposal in voting with two votes and three comments — one visible, one
/// author-deleted, one moderator-hidden — exercises the exact figures the
/// docket endpoint reports per row.
#[tokio::test]
#[allow(clippy::too_many_lines)] // one exhaustive integration scenario; clarity over splitting
async fn docket_summary_tally_and_visible_comment_count() {
    let Some(pool) = setup_pool().await else {
        return;
    };

    let author = make_verified_user(&pool, "sum-author").await;
    let voter = make_verified_user(&pool, "sum-voter").await;

    // Topic + proposal, advanced to deliberation so comments are in-phase.
    let mut tx = pool.begin().await.unwrap();
    let topic = topics::create(
        &mut tx,
        author.id,
        topics::NewTopic {
            slug: &unique("sum-topic"),
            name: "Summary topic",
            description: "",
        },
    )
    .await
    .unwrap();
    let proposal = proposals::create(
        &mut tx,
        proposals::NewProposal {
            topic_id: topic.id,
            author_id: author.id,
            title: "Summarised proposal",
            summary: "Short.",
            body: "Body.",
        },
    )
    .await
    .unwrap();
    proposals::transition_status(
        &mut tx,
        author.id,
        proposal.id,
        ProposalStatus::Deliberation,
        None,
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();

    // Three comments: keep one, delete one, hide one.
    let mut tx = pool.begin().await.unwrap();
    let mk = |body: &'static str| comments::NewComment {
        proposal_id: proposal.id,
        author_id: author.id,
        parent_id: None,
        body,
        stance: Stance::Support,
    };
    let _visible = comments::create(&mut tx, mk("Keep me.")).await.unwrap();
    let to_delete = comments::create(&mut tx, mk("Delete me.")).await.unwrap();
    let to_hide = comments::create(&mut tx, mk("Hide me.")).await.unwrap();
    tx.commit().await.unwrap();

    let mut tx = pool.begin().await.unwrap();
    comments::delete_by_author(&mut tx, author.id, to_delete.id)
        .await
        .unwrap();
    comments::hide_by_moderator(&mut tx, author.id, to_hide.id, "off-topic")
        .await
        .unwrap();
    tx.commit().await.unwrap();

    // list_thread returns every row (including soft-deleted / hidden); the
    // endpoint counts only the visible ones, as asserted below.
    let thread = comments::list_thread(&pool, proposal.id).await.unwrap();
    assert_eq!(
        thread.len(),
        3,
        "all rows returned regardless of visibility"
    );
    let visible_count = thread
        .iter()
        .filter(|c| c.deleted_at.is_none() && c.hidden_at.is_none())
        .count();
    assert_eq!(visible_count, 1, "only the untouched comment is visible");

    // Move to voting and cast one vote each way.
    let mut tx = pool.begin().await.unwrap();
    let now = Utc::now();
    proposals::transition_status(
        &mut tx,
        author.id,
        proposal.id,
        ProposalStatus::Voting,
        Some((now - Duration::seconds(1), now + Duration::days(1))),
    )
    .await
    .unwrap();
    votes::record(&mut tx, proposal.id, author.id, VoteChoice::Yes)
        .await
        .unwrap();
    votes::record(&mut tx, proposal.id, voter.id, VoteChoice::No)
        .await
        .unwrap();
    tx.commit().await.unwrap();

    let active = votes::load_active_for_proposal(&pool, proposal.id)
        .await
        .unwrap();
    assert_eq!(active.len(), 2, "one active vote per voter");

    let dels = civitas_db::delegations::load_active_for_topic(&pool, topic.id)
        .await
        .unwrap();
    let eligible = eligibility::load_eligible_users(&pool, EligibilityPolicy::EmailVerified)
        .await
        .unwrap();
    let scoped: Vec<_> = eligible
        .into_iter()
        .filter(|u| [author.id, voter.id].contains(&u.user_id))
        .collect();

    let result = tally(proposal.id, topic.id, &active, &dels, &scoped);
    assert_eq!(result.yes, Weight::ONE, "author voted yes");
    assert_eq!(result.no, Weight::ONE, "voter voted no");
    assert_eq!(result.abstain, Weight::ZERO);
}

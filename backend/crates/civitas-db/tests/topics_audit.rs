//! End-to-end integration tests against a live Postgres.
//!
//! Skipped silently when `DATABASE_URL` is not set, so `cargo test` is
//! always green in offline environments. CI sets the env explicitly.

use chrono::{Duration, Utc};
use uuid::Uuid;

use civitas_core::{eligibility::EligibilityPolicy, tally};
use civitas_db::{
    audit, comments, delegations, eligibility, proposals, topics, users, votes, DbError,
};
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

#[tokio::test]
async fn end_to_end_topic_proposal_vote_and_tally() {
    let Some(pool) = setup_pool().await else {
        return;
    };

    // Two verified users: one author, one voter.
    let alice = make_verified_user(&pool, "alice").await;
    let bob = make_verified_user(&pool, "bob").await;

    // Topic.
    let mut tx = pool.begin().await.unwrap();
    let topic = topics::create(
        &mut tx,
        alice.id,
        topics::NewTopic {
            slug: &unique("topic"),
            name: "Test topic",
            description: "from end_to_end_topic_proposal_vote_and_tally",
        },
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();

    assert_eq!(topic.name, "Test topic");

    // Proposal.
    let mut tx = pool.begin().await.unwrap();
    let proposal = proposals::create(
        &mut tx,
        proposals::NewProposal {
            topic_id: topic.id,
            author_id: alice.id,
            title: "Adopt the test policy",
            summary: "Short version.",
            body: "Long markdown body.",
        },
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();

    assert_eq!(proposal.status, ProposalStatus::Draft);

    // Move it to deliberation, then voting (one transition at a time).
    let mut tx = pool.begin().await.unwrap();
    proposals::transition_status(
        &mut tx,
        alice.id,
        proposal.id,
        ProposalStatus::Deliberation,
        None,
    )
    .await
    .unwrap();
    let now = Utc::now();
    proposals::transition_status(
        &mut tx,
        alice.id,
        proposal.id,
        ProposalStatus::Voting,
        Some((now - Duration::seconds(1), now + Duration::days(1))),
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();

    // Both users vote.
    let mut tx = pool.begin().await.unwrap();
    votes::record(&mut tx, proposal.id, alice.id, VoteChoice::Yes)
        .await
        .unwrap();
    votes::record(&mut tx, proposal.id, bob.id, VoteChoice::No)
        .await
        .unwrap();
    tx.commit().await.unwrap();

    // Bob changes his mind to abstain — append-only inserts a new row.
    let mut tx = pool.begin().await.unwrap();
    votes::record(&mut tx, proposal.id, bob.id, VoteChoice::Abstain)
        .await
        .unwrap();
    tx.commit().await.unwrap();

    // Tally via civitas-core, fed by civitas-db loaders.
    let active = votes::load_active_for_proposal(&pool, proposal.id)
        .await
        .unwrap();
    assert_eq!(active.len(), 2, "one active vote per voter");

    let dels = delegations::load_active_for_topic(&pool, topic.id)
        .await
        .unwrap();
    let eligible = eligibility::load_eligible_users(&pool, EligibilityPolicy::EmailVerified)
        .await
        .unwrap();

    let result = tally(proposal.id, topic.id, &active, &dels, &eligible);
    assert_eq!(result.yes, Weight::ONE, "alice -> yes");
    assert_eq!(result.abstain, Weight::ONE, "bob's latest vote is abstain");
    assert_eq!(result.no, Weight::ZERO);

    // Audit log should have entries for: topic.created, proposal.created,
    // proposal.status_changed (×2), vote.cast (×3), user.email_verified (×2),
    // user.registered (×2), at minimum.
    let audit_count = sqlx::query_scalar!(
        r#"select count(*) as "n!" from audit_log where actor_id in ($1, $2)"#,
        alice.id.into_inner(),
        bob.id.into_inner(),
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(
        audit_count >= 9,
        "expected at least 9 audit rows, got {audit_count}"
    );
    let _ = audit::Action::VoteCast; // exercise the import
}

#[tokio::test]
async fn delegation_carries_weight_and_cycles_rejected() {
    let Some(pool) = setup_pool().await else {
        return;
    };

    let alice = make_verified_user(&pool, "del-a").await;
    let bob = make_verified_user(&pool, "del-b").await;
    let carol = make_verified_user(&pool, "del-c").await;

    // Topic and proposal in voting.
    let mut tx = pool.begin().await.unwrap();
    let topic = topics::create(
        &mut tx,
        alice.id,
        topics::NewTopic {
            slug: &unique("dt"),
            name: "Delegation topic",
            description: "",
        },
    )
    .await
    .unwrap();
    let proposal = proposals::create(
        &mut tx,
        proposals::NewProposal {
            topic_id: topic.id,
            author_id: alice.id,
            title: "Delegated proposal",
            summary: "...",
            body: "...",
        },
    )
    .await
    .unwrap();
    proposals::transition_status(
        &mut tx,
        alice.id,
        proposal.id,
        ProposalStatus::Deliberation,
        None,
    )
    .await
    .unwrap();
    let now = Utc::now();
    proposals::transition_status(
        &mut tx,
        alice.id,
        proposal.id,
        ProposalStatus::Voting,
        Some((now - Duration::seconds(1), now + Duration::days(1))),
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();

    // alice → bob, bob → carol. Carol votes yes.
    let mut tx = pool.begin().await.unwrap();
    delegations::create_with_cycle_check(&mut tx, alice.id, bob.id, topic.id)
        .await
        .unwrap();
    delegations::create_with_cycle_check(&mut tx, bob.id, carol.id, topic.id)
        .await
        .unwrap();
    votes::record(&mut tx, proposal.id, carol.id, VoteChoice::Yes)
        .await
        .unwrap();
    tx.commit().await.unwrap();

    let active = votes::load_active_for_proposal(&pool, proposal.id)
        .await
        .unwrap();
    let dels = delegations::load_active_for_topic(&pool, topic.id)
        .await
        .unwrap();
    let eligible = eligibility::load_eligible_users(&pool, EligibilityPolicy::EmailVerified)
        .await
        .unwrap();

    // We've created 3 verified users plus any leakage from other tests; filter.
    let scoped: Vec<_> = eligible
        .into_iter()
        .filter(|u| [alice.id, bob.id, carol.id].contains(&u.user_id))
        .collect();

    let result = tally(proposal.id, topic.id, &active, &dels, &scoped);
    assert_eq!(
        result.yes,
        Weight::from(3u32),
        "alice + bob + carol all → yes via delegation"
    );

    // Cycle rejection: trying to add carol → alice would close a cycle.
    let mut tx = pool.begin().await.unwrap();
    let result = delegations::create_with_cycle_check(&mut tx, carol.id, alice.id, topic.id).await;
    match result {
        Err(DbError::DelegationCyclic) => {}
        other => panic!("expected DelegationCyclic, got {other:?}"),
    }
}

#[tokio::test]
async fn comments_thread_and_audit() {
    let Some(pool) = setup_pool().await else {
        return;
    };

    let author = make_verified_user(&pool, "c-author").await;

    let mut tx = pool.begin().await.unwrap();
    let topic = topics::create(
        &mut tx,
        author.id,
        topics::NewTopic {
            slug: &unique("ct"),
            name: "Comments topic",
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
            title: "Discuss this",
            summary: "Discussion proposal",
            body: "A proposal body.",
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
    let top = comments::create(
        &mut tx,
        comments::NewComment {
            proposal_id: proposal.id,
            author_id: author.id,
            parent_id: None,
            body: "I support this.",
            stance: Stance::Support,
        },
    )
    .await
    .unwrap();
    let _reply = comments::create(
        &mut tx,
        comments::NewComment {
            proposal_id: proposal.id,
            author_id: author.id,
            parent_id: Some(top.id),
            body: "Replying to my own thought.",
            stance: Stance::Neutral,
        },
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();

    let thread = comments::list_thread(&pool, proposal.id).await.unwrap();
    assert_eq!(thread.len(), 2);
    assert_eq!(thread[0].parent_id, None);
    assert_eq!(thread[1].parent_id, Some(top.id));
}

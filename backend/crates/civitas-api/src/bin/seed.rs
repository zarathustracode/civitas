//! Development seed script.
//!
//! Populates a fresh database with a small set of users, a topic, and a
//! proposal in deliberation. Idempotent: running twice has no effect (it
//! checks for the canonical seed email before creating).
//!
//! Usage:
//!
//! ```text
//! DATABASE_URL=postgres://... cargo run -p civitas-api --bin seed
//! ```
//!
//! **Do not run against production.** The hard-coded passwords are
//! development-only.

use std::env;

use chrono::Utc;
use civitas_auth::password;
use civitas_db::{
    proposals::{self, NewProposal},
    topics::{self, NewTopic},
    users::{self, NewUser},
};
use civitas_types::ProposalStatus;

const SEED_PASSWORD: &str = "civitas-dev-pw-v1";

#[derive(Clone, Copy)]
struct SeedUser {
    email: &'static str,
    display_name: &'static str,
}

const USERS: &[SeedUser] = &[
    SeedUser {
        email: "alice@example.com",
        display_name: "Alice",
    },
    SeedUser {
        email: "bob@example.com",
        display_name: "Bob",
    },
    SeedUser {
        email: "carol@example.com",
        display_name: "Carol (popular delegate)",
    },
    SeedUser {
        email: "dave@example.com",
        display_name: "Dave (proposal author)",
    },
];

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let database_url =
        env::var("DATABASE_URL").map_err(|_| anyhow::anyhow!("DATABASE_URL must be set"))?;

    println!("connecting to {database_url}");
    let pool = civitas_db::connect(&database_url, 5).await?;

    println!("applying migrations");
    civitas_db::migrate(&pool).await?;

    println!("hashing seed password");
    let password_hash = password::hash(SEED_PASSWORD.to_string()).await?;

    let mut created_user_ids = Vec::new();
    for su in USERS {
        if users::find_by_email(&pool, su.email).await?.is_some() {
            println!("  user {} already exists; skipping", su.email);
            continue;
        }

        let mut tx = pool.begin().await?;
        let user = users::create(
            &mut tx,
            NewUser {
                email: su.email,
                password_hash: &password_hash,
                display_name: su.display_name,
            },
        )
        .await?;
        users::mark_email_verified(&mut tx, user.id).await?;
        tx.commit().await?;
        println!("  created {} ({})", su.email, user.id);
        created_user_ids.push(user.id);
    }

    let dave = users::find_by_email(&pool, "dave@example.com")
        .await?
        .expect("dave was just created or already exists");

    let topic_slug = "demo-policy";
    let topic = if let Some(t) = topics::find_by_slug(&pool, topic_slug).await? {
        println!("  topic '{topic_slug}' already exists; reusing");
        t
    } else {
        let mut tx = pool.begin().await?;
        let t = topics::create(
            &mut tx,
            dave.id,
            NewTopic {
                slug: topic_slug,
                name: "Demo policy",
                description: "A demonstration topic created by the seed script.",
            },
        )
        .await?;
        tx.commit().await?;
        println!("  created topic '{topic_slug}' ({})", t.id);
        t
    };

    let existing_proposals = proposals::list_by_topic(&pool, topic.id, None).await?;
    if existing_proposals.is_empty() {
        let mut tx = pool.begin().await?;
        let proposal = proposals::create(
            &mut tx,
            NewProposal {
                topic_id: topic.id,
                author_id: dave.id,
                title: "Adopt the demo policy",
                summary: "A short illustrative proposal seeded for development.",
                body: "## Demo proposal\n\nThis proposal exists so the dev environment has \
                       something to look at. It does nothing real.\n\n- Point one\n- Point two",
            },
        )
        .await?;
        proposals::transition_status(
            &mut tx,
            dave.id,
            proposal.id,
            ProposalStatus::Deliberation,
            None,
        )
        .await?;
        tx.commit().await?;
        println!("  created proposal '{}' ({})", proposal.title, proposal.id);
    } else {
        println!("  proposals already present on topic '{topic_slug}'; skipping");
    }

    println!();
    println!("seed complete — {} new user(s)", created_user_ids.len());
    println!("password for all seed users: {SEED_PASSWORD}");
    println!("seeded at {}", Utc::now().to_rfc3339());

    Ok(())
}

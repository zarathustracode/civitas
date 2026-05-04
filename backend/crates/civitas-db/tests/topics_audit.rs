use civitas_db::{
    connect, create_audit_log_entry, create_topic, list_topics, migrate, NewAuditLogEntry, NewTopic,
};
use serde_json::json;
use uuid::Uuid;

#[tokio::test]
async fn creates_lists_topics_and_records_audit_log() -> Result<(), Box<dyn std::error::Error>> {
    let Ok(database_url) = std::env::var("DATABASE_URL") else {
        return Ok(());
    };

    let pool = connect(&database_url, 5).await?;
    migrate(&pool).await?;

    let suffix = Uuid::now_v7().simple().to_string();
    let slug = format!("topic-{suffix}");

    let topic = create_topic(
        &pool,
        NewTopic {
            slug: &slug,
            name: "Test Topic",
            description: "Created by civitas-db integration tests.",
        },
    )
    .await?;

    assert_eq!(topic.slug, slug);
    assert_eq!(topic.name, "Test Topic");

    let topics = list_topics(&pool).await?;
    assert!(topics.iter().any(|listed| listed.id == topic.id));

    let audit = create_audit_log_entry(
        &pool,
        NewAuditLogEntry {
            actor_id: None,
            action: "topic.created",
            entity_type: "topic",
            entity_id: topic.id.into_inner(),
            metadata: json!({ "slug": topic.slug }),
        },
    )
    .await?;

    assert_eq!(audit.action, "topic.created");
    assert_eq!(audit.entity_type, "topic");
    assert_eq!(audit.entity_id, topic.id.into_inner());
    assert_eq!(audit.metadata["slug"], slug);

    Ok(())
}

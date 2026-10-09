use serde_json::json;
use sqlx::sqlite::SqlitePoolOptions;

use crate::spaces::conversations::contracts::ConversationAttachment;

use super::super::drafts::{parse_draft, serialize_attachments, validate_attachment};

fn node_attachment(locator: serde_json::Value) -> ConversationAttachment {
    ConversationAttachment::Node {
        node_id: "node".into(),
        source_revision: "revision".into(),
        role: "source".into(),
        locator: Some(locator),
    }
}

#[test]
fn draft_write_rejects_attachments_beyond_the_serialized_limit() {
    let attachments = (0..16)
        .map(|_| {
            node_attachment(json!({
                "sourceRevision": "revision",
                "location": { "kind": "node" },
                "quote": { "exact": "x".repeat(64_000) }
            }))
        })
        .collect::<Vec<_>>();
    assert!(attachments
        .iter()
        .all(|attachment| validate_attachment(attachment).is_ok()));
    assert_eq!(
        serialize_attachments(&attachments).unwrap_err(),
        "conversation-draft-invalid"
    );
}

#[test]
fn draft_attachment_validation_uses_owned_locator_semantics_and_field_bounds() {
    let mismatched_locator = node_attachment(json!({
        "sourceRevision": "different",
        "location": { "kind": "node" }
    }));
    assert_eq!(
        validate_attachment(&mismatched_locator).unwrap_err(),
        "conversation-draft-attachment-invalid"
    );
    let invalid_fields = [
        ConversationAttachment::Node {
            node_id: "n".repeat(129),
            source_revision: "revision".into(),
            role: "source".into(),
            locator: None,
        },
        ConversationAttachment::Node {
            node_id: "node".into(),
            source_revision: "r".repeat(2_049),
            role: "source".into(),
            locator: None,
        },
        ConversationAttachment::Node {
            node_id: "node".into(),
            source_revision: "revision".into(),
            role: "r".repeat(129),
            locator: None,
        },
        ConversationAttachment::File {
            path: "p".repeat(32_769),
            source_revision: "revision".into(),
        },
        ConversationAttachment::File {
            path: "/file.txt".into(),
            source_revision: "r".repeat(2_049),
        },
    ];
    assert!(invalid_fields.iter().all(|attachment| {
        validate_attachment(attachment).unwrap_err() == "conversation-draft-attachment-invalid"
    }));
}

#[tokio::test]
async fn draft_load_rejects_oversized_raw_attachments_before_deserialization() {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("database");
    let row = sqlx::query(
        "SELECT ? AS prompt, ? AS attachments, ? AS extensions, ? AS reacquire_file_names",
    )
    .bind("")
    .bind("x".repeat(1_000_001))
    .bind("{}")
    .bind("[]")
    .fetch_one(&pool)
    .await
    .expect("draft row");

    assert_eq!(
        parse_draft("space", "conversation", row).unwrap_err(),
        "conversation-draft-invalid"
    );
}

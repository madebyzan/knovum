use serde_json::json;
use sqlx::sqlite::SqlitePoolOptions;

use crate::spaces::conversations::contracts::ConversationAttachment;

use super::super::drafts::{
    parse_draft, reconcile_node_attachments, serialize_attachments, validate_attachment,
};

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

#[tokio::test]
async fn draft_node_attachments_reconcile_live_revisions_and_drop_deleted_nodes() {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("database");
    crate::spaces::db::migrate(&pool).await.expect("migrations");
    sqlx::query(
        "INSERT INTO spaces(id,kind,title,created_at,updated_at) VALUES('s1','space','S1',1,1)",
    )
    .execute(&pool)
    .await
    .expect("space");
    sqlx::query(
        "INSERT INTO spaces_nodes(id,space_id,kind,initial_x,initial_y,data,created_at,updated_at)
         VALUES('n-fresh','s1','markdown',0,0,'{\"kind\":\"markdown\",\"title\":\"A\",\"text\":\"hi\",\"knowledgeRevision\":\"rev-2\"}',1,1),
               ('n-same','s1','markdown',0,0,'{\"kind\":\"markdown\",\"title\":\"B\",\"text\":\"hi\",\"knowledgeRevision\":\"rev-1\"}',1,1)",
    )
    .execute(&pool)
    .await
    .expect("nodes");

    let locator = json!({
        "sourceRevision": "rev-1",
        "location": { "kind": "node" }
    });
    let input = vec![
        ConversationAttachment::Node {
            node_id: "n-fresh".into(),
            source_revision: "rev-1".into(),
            role: "source".into(),
            locator: Some(locator.clone()),
        },
        ConversationAttachment::Node {
            node_id: "n-same".into(),
            source_revision: "rev-1".into(),
            role: "source".into(),
            locator: Some(locator.clone()),
        },
        ConversationAttachment::Node {
            node_id: "n-deleted".into(),
            source_revision: "rev-1".into(),
            role: "source".into(),
            locator: None,
        },
        ConversationAttachment::File {
            path: "/tmp/doc.pdf".into(),
            source_revision: "file-rev".into(),
        },
    ];

    let mut connection = pool.acquire().await.expect("connection");
    let reconciled = reconcile_node_attachments(&mut connection, "s1", input)
        .await
        .expect("reconcile");
    assert_eq!(reconciled.len(), 3);
    assert!(matches!(
        &reconciled[0],
        ConversationAttachment::Node {
            node_id,
            source_revision,
            locator: None,
            ..
        } if node_id == "n-fresh" && source_revision == "rev-2"
    ));
    assert!(matches!(
        &reconciled[1],
        ConversationAttachment::Node {
            node_id,
            source_revision,
            locator: Some(_),
            ..
        } if node_id == "n-same" && source_revision == "rev-1"
    ));
    assert!(matches!(
        &reconciled[2],
        ConversationAttachment::File { path, .. } if path == "/tmp/doc.pdf"
    ));
}

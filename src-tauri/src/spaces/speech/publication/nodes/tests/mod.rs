use serde_json::json;

use super::{settle, MediaNodeOutput};
use crate::spaces::{
    db,
    derivation::{
        inheritance::DerivedOutput,
        pending_output::{self, PendingOutput},
    },
    nodes,
    speech::contracts::SpeechAudioOutput,
    structure::{self, Size},
};

const STEMS: [&str; 2] = ["vocals", "background"];

fn reserved(stem: &str) -> PendingOutput {
    PendingOutput {
        node_id: stem.into(),
        kind: "audio",
        data: json!({
            "kind":"audio","title":stem,"fileName":format!("{stem}.wav"),
            "assetPath":format!("/managed/reserved-{stem}.wav"),"processingState":"processing"
        }),
        footprint: Size {
            width: 288.0,
            height: 80.0,
        },
    }
}

fn published(stem: &str) -> MediaNodeOutput {
    MediaNodeOutput {
        node_id: stem.into(),
        output: SpeechAudioOutput {
            path: format!("/managed/{stem}.wav"),
            sha256: "b".repeat(64),
            sample_rate: 44_100,
            sample_count: 44_100,
        },
        derived: DerivedOutput::SeparatedBackground,
        cover: None,
    }
}

#[tokio::test]
async fn reserved_outputs_carry_their_lineage_and_settle_without_touching_the_source() {
    let pool = crate::spaces::speech::tests::database().await;
    let mut tx = db::begin_write(&pool).await.unwrap();
    structure::repository::initialize_empty_in_transaction(&mut tx, "space")
        .await
        .unwrap();
    sqlx::query("INSERT INTO spaces_runs(id,scope_kind,home_space_id,principal_kind,principal_id,kind,state,execution_epoch,policy_snapshot,input_snapshot,budget,created_at,updated_at) VALUES('run','space','space','user','local-user','workflow','running',0,'{}','{}','{}',1,1)")
        .execute(&mut *tx).await.unwrap();
    let request = serde_json::from_value(json!({
        "spaceId":"space","mutationId":"source-create",
        "targetParentId":null,"nodes":[{"id":"source","kind":"audio",
            "initialX":0,"initialY":0,"width":288,"height":80,"sizingMode":"content",
            "sourceChunks":[],"data":{"kind":"audio","title":"Source",
                "assetPath":"/managed/source.wav","fileName":"source.wav",
                "sourceRevision":format!("sha256:{}", "a".repeat(64)),"durationSeconds":1}}]
    }))
    .unwrap();
    nodes::tests::insert_many_in_transaction(&mut tx, request, None)
        .await
        .unwrap();
    let source_before: String =
        sqlx::query_scalar("SELECT data FROM spaces_nodes WHERE id='source'")
            .fetch_one(&mut *tx)
            .await
            .unwrap();
    // The Run reserves both tracks as it is created: the canvas shows them while it works.
    pending_output::reserve_in_transaction(
        &mut tx,
        "space",
        STEMS.map(reserved).into(),
        &["source".to_owned()],
        "Track separated from its source media",
        "run",
        None,
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();
    let waiting: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM spaces_reserved_outputs")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(waiting, 2);
    let history_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM spaces_edit_history")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(history_count, 2, "the two reserved nodes are one edit");

    let mut tx = db::begin_write(&pool).await.unwrap();
    settle(&mut tx, "space", "source", &STEMS.map(published), None)
        .await
        .unwrap();
    tx.commit().await.unwrap();
    let data: String = sqlx::query_scalar("SELECT data FROM spaces_nodes WHERE id='vocals'")
        .fetch_one(&pool)
        .await
        .unwrap();
    let data: serde_json::Value = serde_json::from_str(&data).unwrap();
    assert_eq!(data["assetPath"], "/managed/vocals.wav");
    assert_eq!(data["fileName"], "vocals.wav");
    assert_eq!(data["durationSeconds"], 1);
    assert!(data.get("processingState").is_none());
    let waiting: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM spaces_reserved_outputs")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(waiting, 0, "settled outputs wait on nothing");
    let source_after: String =
        sqlx::query_scalar("SELECT data FROM spaces_nodes WHERE id='source'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(
        source_before, source_after,
        "derivation must not edit the source"
    );

    for (redo, expected_nodes, expected_relations) in
        [(false, 1, 0), (true, 3, 2), (false, 1, 0), (true, 3, 2)]
    {
        let mut tx = db::begin_write(&pool).await.unwrap();
        let (snapshot, revision) = structure::repository::load_in_transaction(&mut tx, "space")
            .await
            .unwrap();
        structure::history::apply(&mut tx, "space", revision, &snapshot, redo)
            .await
            .unwrap();
        let active_nodes: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM spaces_nodes WHERE deleted_at IS NULL")
                .fetch_one(&mut *tx)
                .await
                .unwrap();
        let active_relations: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM spaces_relations WHERE kind='derives-from' AND status='active' AND target_id='source'",
        ).fetch_one(&mut *tx).await.unwrap();
        assert_eq!(active_nodes, expected_nodes);
        assert_eq!(active_relations, expected_relations);
        tx.commit().await.unwrap();
    }
}

#[tokio::test]
async fn an_output_that_was_never_reserved_cannot_be_settled() {
    let pool = crate::spaces::speech::tests::database().await;
    let mut tx = db::begin_write(&pool).await.unwrap();
    structure::repository::initialize_empty_in_transaction(&mut tx, "space")
        .await
        .unwrap();
    let request = serde_json::from_value(json!({
        "spaceId":"space","mutationId":"source-create","targetParentId":null,
        "nodes":[{"id":"source","kind":"audio","initialX":0,"initialY":0,"width":288,"height":80,
            "sizingMode":"content","sourceChunks":[],"data":{"kind":"audio","title":"Source",
                "assetPath":"/managed/source.wav","fileName":"source.wav",
                "sourceRevision":format!("sha256:{}", "a".repeat(64))}}]
    }))
    .unwrap();
    nodes::tests::insert_many_in_transaction(&mut tx, request, None)
        .await
        .unwrap();
    assert!(
        settle(&mut tx, "space", "source", &[published("vocals")], None)
            .await
            .is_err()
    );
}

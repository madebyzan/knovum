use super::{publish, MediaNodeOutput};
use crate::spaces::{db, nodes, speech::contracts::SpeechAudioOutput, structure};
use serde_json::json;

#[tokio::test]
async fn derived_pair_undo_and_redo_preserve_relations_and_source() {
    let pool = crate::spaces::speech::tests::database().await;
    let mut tx = db::begin_write(&pool).await.unwrap();
    structure::repository::initialize_empty_in_transaction(&mut tx, "space")
        .await
        .unwrap();
    let request = serde_json::from_value(json!({
        "spaceId":"space","mutationId":"source-create","expectedEditRevision":0,
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
    let outputs = ["vocals", "background"].map(|stem| MediaNodeOutput {
        node_id: stem.into(),
        kind: "audio",
        title: stem.into(),
        output: SpeechAudioOutput {
            path: format!("/managed/{stem}.wav"),
            sha256: "b".repeat(64),
            sample_rate: 44_100,
            sample_count: 44_100,
        },
    });
    publish(&mut tx, "space", "source", &outputs).await.unwrap();
    tx.commit().await.unwrap();
    let history_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM spaces_edit_history")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(history_count, 2, "the two derived nodes are one edit");

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
        let source_after: String = sqlx::query_scalar(
            "SELECT data FROM spaces_nodes WHERE id='source' AND deleted_at IS NULL",
        )
        .fetch_one(&mut *tx)
        .await
        .unwrap();
        assert_eq!(
            source_before, source_after,
            "derivation must not edit the source"
        );
        tx.commit().await.unwrap();
    }
}

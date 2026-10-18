use serde_json::{json, Value};
use sqlx::SqlitePool;
use tokio_util::sync::CancellationToken;

use super::super::{
    persistence, preparation::PreparedSubtitles, start, RenderSubtitledVideoRequest,
};
use crate::spaces::{
    db,
    derivation::{action_execution::ExecutionAuthority, video_output::RenderedVideo},
    knowledge::model::revision,
    nodes::contracts::InsertNodesRequest,
    previews::GeneratedPreview,
    structure::repository,
};

async fn fixture(with_transcript: bool) -> (SqlitePool, RenderSubtitledVideoRequest, Value) {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    db::migrate(&pool).await.unwrap();
    sqlx::query("INSERT INTO spaces(id,kind,title,created_at,updated_at) VALUES('space','space','Space',1,1)").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO spaces_runs(id,scope_kind,home_space_id,principal_kind,principal_id,kind,state,execution_epoch,policy_snapshot,input_snapshot,budget,created_at,updated_at) VALUES('job','space','space','user','local-user','action','running',0,'{}','{}','{}',1,1)").execute(&pool).await.unwrap();
    let mut source = json!({"kind":"video","title":"Interview","assetPath":"/owned/source.mp4","fileName":"source.mp4","sourceRevision":"asset:1",
        "durationSeconds":42,
        "mediaAnalysis":{"schemaVersion":1,"sourceRevision":"asset:1","durationMs":42_000,"waveform":[0.5]}});
    if with_transcript {
        source["mediaKnowledge"] = crate::media::transcript::tests::knowledge_fixture(
            "asset:1",
            vec![crate::media::transcript::TranscriptSegment {
                start_ms: 0,
                end_ms: 1000,
                text: "Hello".into(),
            }],
        );
    }
    revision::apply(&mut source).unwrap();
    let mut output = json!({"kind":"video","title":"Subtitled","fileName":"subtitled.mp4","assetPath":"/owned/pending.mp4","processingState":"processing"});
    revision::apply(&mut output).unwrap();
    let request = RenderSubtitledVideoRequest {
        id: "job".into(),
        space_id: "space".into(),
        source_node_id: "source".into(),
        output_node_id: "output".into(),
        source_revision: "asset:1".into(),
        knowledge_revision: source["knowledgeRevision"].as_str().unwrap().into(),
    };
    let mut tx = db::begin_write(&pool).await.unwrap();
    repository::initialize_empty_in_transaction(&mut tx, "space")
        .await
        .unwrap();
    let nodes: InsertNodesRequest = serde_json::from_value(json!({
        "spaceId":"space","mutationId":"create","targetParentId":null,
        "nodes":[
            {"id":"source","kind":"video","initialX":0,"initialY":0,"width":300,"height":200,"sizingMode":"frame","data":source,"sourceChunks":[]},
            {"id":"output","kind":"video","initialX":400,"initialY":0,"width":300,"height":200,"sizingMode":"frame","data":output,"sourceChunks":[]}
        ]
    })).unwrap();
    crate::spaces::nodes::tests::insert_many_in_transaction(&mut tx, nodes, None)
        .await
        .unwrap();
    tx.commit().await.unwrap();
    (pool, request, source)
}

fn authority() -> ExecutionAuthority {
    ExecutionAuthority {
        run_id: "job".into(),
        execution_epoch: 0,
    }
}

fn rendered() -> RenderedVideo {
    RenderedVideo {
        path: "/owned/subtitled.mp4".into(),
        size: 100,
        revision: "asset:output".into(),
        cover: Some(GeneratedPreview {
            path: "/previews/subtitled.png".into(),
            width: 640,
            height: 360,
        }),
    }
}

#[tokio::test]
async fn automatic_subtitles_publish_only_a_derived_video_and_leave_source_unchanged() {
    let (pool, request, original) = fixture(true).await;
    let source = start::start(&pool, &request, &authority()).await.unwrap();
    assert!(matches!(source.subtitles, PreparedSubtitles::Ready(srt) if srt.contains("Hello")));
    let completion = persistence::complete(
        &pool,
        "job",
        rendered(),
        &authority(),
        &CancellationToken::new(),
        None,
    )
    .await
    .unwrap();
    assert_eq!(completion.data["sourceRevision"], "asset:output");
    assert!(completion.data.get("processingState").is_none());
    // The picture is new, so it carries its own cover and the geometry that cover reports.
    assert_eq!(completion.data["previewPath"], "/previews/subtitled.png");
    assert_eq!(completion.data["intrinsicWidth"], 640);
    // The timeline and what is said in it did not change, so they are carried over and re-pinned
    // instead of being measured and recognized again.
    assert_eq!(completion.data["durationSeconds"], 42);
    assert_eq!(completion.data["mediaAnalysis"]["durationMs"], 42_000);
    assert_eq!(
        completion.data["mediaAnalysis"]["sourceRevision"],
        "asset:output"
    );
    assert_eq!(
        completion.data["mediaKnowledge"]["sourceRevision"],
        "asset:output"
    );
    assert!(
        completion.data["mediaKnowledge"]["transcript"]["subtitleCues"][0]["text"]
            .as_str()
            .is_some_and(|text| text.contains("Hello"))
    );
    // Nothing carried over is read again: the only work left is understanding the new picture.
    let queued: Vec<String> = sqlx::query_scalar(
        "SELECT job_type FROM spaces_enrichment_jobs WHERE node_id='output' ORDER BY job_type",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(queued, ["analyze_visual"]);
    // Lineage was written when the output was reserved, so publication only settles content.
    let counts: (i64, i64, i64) = sqlx::query_as("SELECT (SELECT COUNT(*) FROM spaces_nodes),(SELECT COUNT(*) FROM spaces_nodes WHERE kind='subtitle'),(SELECT COUNT(*) FROM spaces_relations)").fetch_one(&pool).await.unwrap();
    assert_eq!(counts, (2, 0, 0));
    let encoded: String = sqlx::query_scalar("SELECT data FROM spaces_nodes WHERE id='source'")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(serde_json::from_str::<Value>(&encoded).unwrap(), original);
}

#[tokio::test]
async fn untranscribed_video_requires_a_qualified_recognition_engine() {
    let (pool, request, _) = fixture(false).await;
    let engine = crate::spaces::assets::transcription::engine_id();
    let completion = start::start(&pool, &request, &authority()).await;
    match engine {
        Ok(_) => {
            let source = completion.unwrap();
            assert!(matches!(source.subtitles, PreparedSubtitles::Recognize));
            let row: (String, i64) = sqlx::query_as("SELECT action_kind,(SELECT COUNT(*) FROM spaces_knowledge_derivation_inputs WHERE derivation_id='job') FROM spaces_knowledge_derivations WHERE id='job'").fetch_one(&pool).await.unwrap();
            assert_eq!(row, ("add-video-subtitles".into(), 1));
        }
        Err(error) => {
            assert_eq!(completion.err().unwrap(), error);
            let count: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM spaces_knowledge_derivations WHERE id='job'",
            )
            .fetch_one(&pool)
            .await
            .unwrap();
            assert_eq!(count, 0);
        }
    }
}

#[tokio::test]
async fn source_change_and_cancellation_prevent_publication() {
    for change_source in [true, false] {
        let (pool, request, mut source) = fixture(true).await;
        start::start(&pool, &request, &authority()).await.unwrap();
        let cancellation = CancellationToken::new();
        if change_source {
            source["mediaKnowledge"]["transcript"]["subtitleCues"][0]["text"] = json!("Edited");
            revision::apply(&mut source).unwrap();
            sqlx::query("UPDATE spaces_nodes SET data=$1 WHERE id='source'")
                .bind(source.to_string())
                .execute(&pool)
                .await
                .unwrap();
        } else {
            cancellation.cancel();
        }
        assert!(
            persistence::complete(&pool, "job", rendered(), &authority(), &cancellation, None)
                .await
                .is_err()
        );
        let output: String = sqlx::query_scalar("SELECT data FROM spaces_nodes WHERE id='output'")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(
            serde_json::from_str::<Value>(&output).unwrap()["processingState"],
            "processing"
        );
        let relations: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM spaces_relations")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(relations, 0);
    }
}

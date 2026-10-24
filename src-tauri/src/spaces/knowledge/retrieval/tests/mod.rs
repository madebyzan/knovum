use super::hybrid::reciprocal_rank_fusion;
use super::{generation, index_state, index_writer::tests::fixture::Fixture, indexing_control};
use tokio_util::sync::CancellationToken;

fn matched(id: &str, score: f64) -> serde_json::Value {
    serde_json::json!({ "chunkId": id, "score": score })
}

#[test]
fn reciprocal_rank_fusion_deduplicates_chunks() {
    let values = reciprocal_rank_fusion(
        vec![matched("a", 7.0), matched("b", 4.0)],
        vec![matched("b", 8.0), matched("a", 3.0), matched("c", 1.0)],
    );
    let ids = values
        .iter()
        .filter_map(|value| value["chunkId"].as_str())
        .collect::<Vec<_>>();
    assert_eq!(ids.len(), 3);
    assert!(ids.contains(&"a") && ids.contains(&"b") && ids.contains(&"c"));
}

#[test]
fn reciprocal_rank_fusion_keeps_both_score_payloads() {
    let values = reciprocal_rank_fusion(vec![matched("a", 7.0)], vec![matched("a", 3.0)]);
    assert_eq!(values.len(), 1);
    assert_eq!(values[0]["lexicalScore"], 7.0);
    assert_eq!(values[0]["semanticScore"], 3.0);
}

#[tokio::test]
async fn a_rebuild_keeps_the_live_index_until_the_new_generation_is_complete() {
    let fixture = Fixture::new().await;
    sqlx::query(
        "UPDATE spaces_retrieval_generations SET status='active',activated_at=1 WHERE id='generation'",
    )
    .execute(&fixture.pool)
    .await
    .unwrap();

    let rebuilt = generation::begin_rebuild(&fixture.pool, "space", "spec")
        .await
        .unwrap();
    assert_ne!(rebuilt.id, "generation");
    // Re-encoding every chunk of a Space takes as long as it takes. Searches keep answering from
    // the generation that is still complete for the whole build.
    assert_eq!(
        generation::active(&fixture.pool, "space", "spec")
            .await
            .unwrap()
            .id,
        "generation"
    );

    // A generation is publishable once it has encoded everything the Space currently offers —
    // here the image's description and the node title the structure contributes.
    sqlx::query(
        "INSERT INTO spaces_chunk_embeddings(\
           generation_id,chunk_id,source_revision,embedding_spec_id,dimensions,updated_at\
         ) SELECT $1,id,source_revision,'spec',2,1 \
           FROM spaces_current_knowledge_chunks WHERE space_id='space'",
    )
    .bind(&rebuilt.id)
    .execute(&fixture.pool)
    .await
    .unwrap();
    generation::activate(&fixture.pool, "space", &rebuilt)
        .await
        .unwrap();

    assert_eq!(
        generation::active(&fixture.pool, "space", "spec")
            .await
            .unwrap()
            .id,
        rebuilt.id
    );
    let replaced: String =
        sqlx::query_scalar("SELECT status FROM spaces_retrieval_generations WHERE id='generation'")
            .fetch_one(&fixture.pool)
            .await
            .unwrap();
    assert_eq!(replaced, "retired");
    fixture.pool.close().await;
}

#[tokio::test]
async fn distinguishes_unprepared_generations_from_database_failures() {
    let fixture = Fixture::new().await;
    assert!(generation::find_active(&fixture.pool, "space", "spec")
        .await
        .unwrap()
        .is_none());
    assert_eq!(
        generation::active(&fixture.pool, "space", "spec")
            .await
            .unwrap_err(),
        generation::REBUILD_REQUIRED
    );

    sqlx::query("UPDATE spaces_retrieval_generations SET status='active',activated_at=1 WHERE id='generation'")
        .execute(&fixture.pool).await.unwrap();
    assert_eq!(
        generation::find_active(&fixture.pool, "space", "spec")
            .await
            .unwrap()
            .unwrap()
            .id,
        "generation"
    );
    assert!(
        generation::find_active(&fixture.pool, "space", "different-spec")
            .await
            .unwrap()
            .is_none()
    );
    assert_eq!(
        generation::active(&fixture.pool, "space", "different-spec")
            .await
            .unwrap_err(),
        generation::REBUILD_REQUIRED
    );

    fixture.pool.close().await;
    assert!(generation::find_active(&fixture.pool, "space", "spec")
        .await
        .is_err());
}

#[tokio::test]
async fn a_space_has_one_indexing_pass_at_a_time() {
    let space = format!("space-{}", uuid::Uuid::new_v4());
    let pass = indexing_control::begin(
        &space,
        indexing_control::Admission::Requested,
        CancellationToken::new(),
    )
    .await
    .unwrap();
    assert!(indexing_control::is_running(&space).unwrap());

    // A second request would read the same unindexed chunks, spend the one embedding worker on
    // them twice, and race the first pass to publish a generation. The running pass is already
    // doing that work, so the request is declined rather than duplicated.
    assert_eq!(
        indexing_control::begin(
            &space,
            indexing_control::Admission::Requested,
            CancellationToken::new(),
        )
        .await
        .err(),
        Some(indexing_control::BUSY.to_owned())
    );

    // Cancelling names the Space, because that is what a pass belongs to.
    assert!(indexing_control::cancel(&space).unwrap());
    assert!(pass.cancellation().is_cancelled());

    drop(pass);
    assert!(!indexing_control::is_running(&space).unwrap());
    assert!(!indexing_control::cancel(&space).unwrap());
}

#[tokio::test]
async fn work_that_needs_the_index_waits_for_the_running_pass() {
    let space = format!("space-{}", uuid::Uuid::new_v4());
    let requested = indexing_control::begin(
        &space,
        indexing_control::Admission::Requested,
        CancellationToken::new(),
    )
    .await
    .unwrap();
    let cancellation = CancellationToken::new();
    let waiting = tokio::spawn({
        let space = space.clone();
        let cancellation = cancellation.clone();
        async move {
            indexing_control::begin(&space, indexing_control::Admission::Required, cancellation)
                .await
                .map(|pass| pass.cancellation().clone())
        }
    });

    assert!(!waiting.is_finished());
    drop(requested);

    // A Run cannot answer until the Space is searchable, so it takes its turn instead of failing,
    // and stays cancellable through the Run that needs it rather than through the Space.
    let taken = waiting.await.unwrap().unwrap();
    assert!(!taken.is_cancelled());
    assert!(!indexing_control::cancel(&space).unwrap());
}

#[tokio::test]
async fn index_state_reads_what_a_search_can_reach_today() {
    let fixture = Fixture::new().await;

    // A Space whose only generation is still being built answers nothing, and the whole Space —
    // the image's description and the node title the structure contributes — is the work left.
    let building = index_state::measure(&fixture.pool, "space", "spec", true, false)
        .await
        .unwrap();
    assert_eq!(
        (building.total, building.indexed, building.pending),
        (2, 0, 2)
    );
    assert!(!building.searchable && building.rebuilding);
    assert_eq!(building.indexed_at, None);

    sqlx::query(
        "INSERT INTO spaces_chunk_embeddings(\
           generation_id,chunk_id,source_revision,embedding_spec_id,dimensions,updated_at\
         ) SELECT 'generation',id,source_revision,'spec',2,1 \
           FROM spaces_current_knowledge_chunks WHERE space_id='space'",
    )
    .execute(&fixture.pool)
    .await
    .unwrap();
    generation::activate(
        &fixture.pool,
        "space",
        &generation::find_building(&fixture.pool, "space", "spec")
            .await
            .unwrap()
            .unwrap(),
    )
    .await
    .unwrap();

    let live = index_state::measure(&fixture.pool, "space", "spec", true, false)
        .await
        .unwrap();
    assert_eq!((live.total, live.indexed, live.pending), (2, 2, 0));
    assert!(live.searchable && !live.rebuilding);
    assert!(live.indexed_at.is_some());

    // The vector table is the index; markers alone are a complete-looking index that cannot
    // answer, and the only honest reading of it is a Space the next pass re-encodes whole.
    let lost = index_state::measure(&fixture.pool, "space", "spec", false, true)
        .await
        .unwrap();
    assert_eq!((lost.total, lost.indexed, lost.pending), (2, 0, 2));
    assert!(!lost.searchable && lost.rebuilding && lost.indexing);

    fixture.pool.close().await;
}

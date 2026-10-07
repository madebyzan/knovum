use super::search;
use crate::spaces::knowledge::retrieval::{
    generation::{self, IndexGeneration},
    index_writer::{publish_embeddings, tests::fixture::Fixture, KnowledgeEmbeddingInput},
};

async fn indexed_spaces() -> Fixture {
    let fixture = Fixture::new().await;
    index_space(&fixture, "space", "generation", &[0.0, 1.0]).await;
    sqlx::raw_sql("UPDATE spaces SET updated_at=100 WHERE id='space';
        INSERT INTO spaces(id,kind,title,created_at,updated_at) VALUES('other','space','Other',1,1);
        INSERT INTO spaces_nodes(id,space_id,kind,initial_x,initial_y,data,created_at,updated_at)
        VALUES('other-node','other','image',0,0,'{\"kind\":\"image\",\"title\":\"Other image\",\"sourceRevision\":\"r1\"}',1,1);
        INSERT INTO spaces_knowledge_chunks(id,space_id,node_id,origin,kind,source_revision,text,locator,created_at,updated_at)
        VALUES('other-chunk','other','other-node','source','image_description','r1','Relevant description','{}',1,1);
        INSERT INTO spaces_retrieval_generations(id,space_id,embedding_spec_id,status,created_at)
        VALUES('other-generation','other','spec','building',1);")
        .execute(&fixture.pool).await.unwrap();
    index_space(&fixture, "other", "other-generation", &[1.0, 0.0]).await;
    fixture
}

async fn index_space(fixture: &Fixture, space_id: &str, generation_id: &str, source: &[f64]) {
    let chunks: Vec<(String, String, String)> = sqlx::query_as(
        "SELECT id,source_revision,origin FROM spaces_current_knowledge_chunks WHERE space_id=$1",
    )
    .bind(space_id)
    .fetch_all(&fixture.pool)
    .await
    .unwrap();
    assert_eq!(chunks.len(), 2);
    let input = chunks
        .into_iter()
        .map(
            |(chunk_id, source_revision, origin)| KnowledgeEmbeddingInput {
                generation_id: generation_id.into(),
                embedding_spec_id: "spec".into(),
                chunk_id,
                source_revision,
                dimensions: 2,
                vector: if origin == "source" {
                    source.to_vec()
                } else {
                    vec![-1.0, 0.0]
                },
            },
        )
        .collect();
    publish_embeddings(&fixture.pool, &fixture.index, input)
        .await
        .unwrap();
    generation::activate(
        &fixture.pool,
        space_id,
        &IndexGeneration {
            id: generation_id.into(),
            embedding_spec_id: "spec".into(),
        },
    )
    .await
    .unwrap();
}

#[tokio::test]
async fn ranks_globally_by_similarity_instead_of_space_recency() {
    let fixture = indexed_spaces().await;
    let values = search(&fixture.pool, &fixture.index, &[1.0, 0.0], "spec", 80)
        .await
        .unwrap();
    assert_eq!(values.len(), 4);
    assert_eq!(
        values
            .iter()
            .take(2)
            .map(|value| value["chunkId"].as_str().unwrap())
            .collect::<Vec<_>>(),
        ["other-chunk", "c1"]
    );
    assert_eq!(values[0]["spaceTitle"], "Other");
    assert_eq!(values[1]["spaceTitle"], "Space");
    let limited = search(&fixture.pool, &fixture.index, &[1.0, 0.0], "spec", 1)
        .await
        .unwrap();
    assert_eq!(limited.len(), 1);
    assert_eq!(limited[0]["spaceId"], "other");
}

#[tokio::test]
async fn excludes_stale_chunks_and_archived_spaces() {
    let fixture = indexed_spaces().await;
    sqlx::query(
        "UPDATE spaces_nodes SET data=json_set(data,'$.sourceRevision','r2') WHERE id='other-node'",
    )
    .execute(&fixture.pool)
    .await
    .unwrap();
    let values = search(&fixture.pool, &fixture.index, &[1.0, 0.0], "spec", 80)
        .await
        .unwrap();
    assert_eq!(values.len(), 3);
    assert!(!values.iter().any(|value| value["chunkId"] == "other-chunk"));
    assert_eq!(values[0]["spaceId"], "space");
    sqlx::query("UPDATE spaces SET archived_at=2 WHERE id='other'")
        .execute(&fixture.pool)
        .await
        .unwrap();
    let values = search(&fixture.pool, &fixture.index, &[1.0, 0.0], "spec", 80)
        .await
        .unwrap();
    assert_eq!(values.len(), 2);
    assert!(values.iter().all(|value| value["spaceId"] == "space"));
}

#[tokio::test]
async fn rejects_missing_or_incompatible_active_generations() {
    let fixture = indexed_spaces().await;
    assert_eq!(
        search(
            &fixture.pool,
            &fixture.index,
            &[1.0, 0.0],
            "different-spec",
            80
        )
        .await
        .unwrap_err(),
        generation::REBUILD_REQUIRED
    );
    sqlx::query("UPDATE spaces_retrieval_generations SET status='retired',activated_at=NULL WHERE space_id='other'")
        .execute(&fixture.pool)
        .await
        .unwrap();
    assert_eq!(
        search(&fixture.pool, &fixture.index, &[1.0, 0.0], "spec", 80)
            .await
            .unwrap_err(),
        generation::REBUILD_REQUIRED
    );
}

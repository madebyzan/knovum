use serde_json::json;
use sqlx::SqlitePool;

use super::resolve;
use crate::spaces::{
    db,
    derivation::inputs::{self, DerivationInput, RevisionBasis},
    knowledge::model::revision,
    nodes::tests::insert_source_in_transaction,
    structure::repository,
};

/// A completed `action` that read `source` as it was edited then, and published `output`.
async fn derived(action: &str) -> SqlitePool {
    let pool = crate::spaces::derivation::tests::running_action_pool("job").await;
    let mut source = json!({"kind":"image","title":"Source","fileName":"source.png",
        "assetPath":"/owned/source.png","sourceRevision":"asset:1","imageEdit":crate::images::quarter_turn_edit()});
    revision::apply(&mut source).unwrap();
    let mut output = json!({"kind":"image","title":"Output","fileName":"output.png",
        "assetPath":"/owned/output.png","sourceRevision":"asset:2"});
    revision::apply(&mut output).unwrap();
    let mut tx = db::begin_write(&pool).await.unwrap();
    repository::initialize_empty_in_transaction(&mut tx, "space")
        .await
        .unwrap();
    insert_source_in_transaction(&mut tx, "space", "source", "image", source)
        .await
        .unwrap();
    insert_source_in_transaction(&mut tx, "space", "output", "image", output)
        .await
        .unwrap();
    sqlx::query("INSERT INTO spaces_knowledge_derivations(id,space_id,output_node_id,action_kind,action_version,engine,status,created_at,updated_at) VALUES('job','space','output',$1,1,'local:test','running',1,1)")
        .bind(action)
        .execute(&mut *tx).await.unwrap();
    inputs::insert(
        &mut tx,
        "job",
        &[DerivationInput {
            node_id: "source",
            role: "primary",
            source_revision: "asset:1",
            revision_basis: RevisionBasis::Rendered,
        }],
    )
    .await
    .unwrap();
    sqlx::query(
        "UPDATE spaces_knowledge_derivations SET status='completed',completed_at=2 WHERE id='job'",
    )
    .execute(&mut *tx)
    .await
    .unwrap();
    tx.commit().await.unwrap();
    pool
}

async fn remove_source_image_edit(pool: &SqlitePool) {
    sqlx::query("UPDATE spaces_nodes SET data=json_remove(data,'$.imageEdit') WHERE id='source'")
        .execute(pool)
        .await
        .unwrap();
}

async fn replace_source_asset(pool: &SqlitePool) {
    sqlx::query(
        "UPDATE spaces_nodes SET data=json_set(data,'$.sourceRevision','asset:3') WHERE id='source'",
    )
    .execute(pool)
    .await
    .unwrap();
}

#[tokio::test]
async fn an_output_compares_with_the_input_it_read_even_after_its_source_is_edited_again() {
    let pool = derived("upscale-image").await;
    remove_source_image_edit(&pool).await;

    let comparable = resolve(&pool, "output").await.unwrap().unwrap();

    assert_eq!(comparable.asset_path, "/owned/source.png");
    assert_eq!(comparable.edit["rotation"].as_f64(), Some(90.0));
    assert!(comparable.input_revision.starts_with("rendered1:"));
}

#[tokio::test]
async fn a_replaced_source_asset_leaves_nothing_honest_to_compare() {
    let pool = derived("remove-image-background").await;
    replace_source_asset(&pool).await;

    assert!(resolve(&pool, "output").await.unwrap().is_none());
}

#[tokio::test]
async fn only_a_transform_of_the_same_picture_compares() {
    for (action, compares) in [
        ("upscale-image", true),
        ("remove-image-object", true),
        ("extract-image-selection", true),
        ("generate-image-depth", false),
        ("generate-image-angle", false),
    ] {
        let pool = derived(action).await;
        assert_eq!(
            resolve(&pool, "output").await.unwrap().is_some(),
            compares,
            "{action}"
        );
    }
}

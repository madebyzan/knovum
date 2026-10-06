use super::transactions::database;
use crate::spaces::{db, structure::portal_integrity};

#[tokio::test]
async fn checks_inbound_references_and_ignores_unrelated_spaces() {
    let pool = database().await;
    sqlx::query("INSERT INTO spaces(id,kind,title,created_at,updated_at) VALUES('other','space','Other',1,1)")
        .execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO spaces_nodes(id,space_id,kind,initial_x,initial_y,data,created_at,updated_at) VALUES('portal','other','portal',0,0,'{\"id\":\"portal\",\"kind\":\"portal\",\"targetSpaceId\":\"other\",\"targetStructureNodeId\":\"missing\"}',1,1)")
        .execute(&pool).await.unwrap();
    let mut tx = db::begin_write(&pool).await.unwrap();
    portal_integrity::validate_structure_targets(&mut tx, &["space"])
        .await
        .unwrap();
    sqlx::query(
        "UPDATE spaces_nodes SET data=json_set(data,'$.targetSpaceId','space') WHERE id='portal'",
    )
    .execute(&mut *tx)
    .await
    .unwrap();
    assert!(
        portal_integrity::validate_structure_targets(&mut tx, &["space"])
            .await
            .unwrap_err()
            .contains("portal")
    );
    sqlx::query("INSERT INTO spaces_structure_nodes(id,space_id,kind,order_index,title) VALUES('missing','space','group',0,'Target')")
        .execute(&mut *tx).await.unwrap();
    portal_integrity::validate_structure_targets(&mut tx, &["space"])
        .await
        .unwrap();
    sqlx::query("UPDATE spaces_structure_nodes SET deleted_at=2 WHERE id='missing'")
        .execute(&mut *tx)
        .await
        .unwrap();
    assert!(
        portal_integrity::validate_structure_targets(&mut tx, &["space"])
            .await
            .is_err()
    );
    tx.rollback().await.unwrap();
}

#[tokio::test]
async fn keeps_permanent_deletion_constraints_scoped_to_the_target() {
    let pool = database().await;
    sqlx::query("INSERT INTO spaces_nodes(id,space_id,kind,initial_x,initial_y,data,created_at,updated_at) VALUES('structure-portal','space','portal',0,0,'{\"id\":\"structure-portal\",\"kind\":\"portal\",\"targetSpaceId\":\"space\",\"targetStructureNodeId\":\"group\"}',1,1),('node-portal','space','portal',0,0,'{\"id\":\"node-portal\",\"kind\":\"portal\",\"targetSpaceId\":\"space\",\"targetNodeId\":\"node\"}',1,1)")
        .execute(&pool).await.unwrap();
    let mut tx = db::begin_write(&pool).await.unwrap();
    let ids = std::collections::HashSet::from(["group"]);
    assert!(
        portal_integrity::reject_structure_targets(&mut tx, "space", &ids)
            .await
            .is_err()
    );
    portal_integrity::reject_structure_targets(&mut tx, "other", &ids)
        .await
        .unwrap();
    assert!(portal_integrity::reject_node_targets(
        &mut tx,
        "space",
        &std::collections::HashSet::from(["node"])
    )
    .await
    .is_err());
}

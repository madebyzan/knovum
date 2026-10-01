
use super::*;
#[test]
fn rebinds_canonical_citations_and_preserves_detached_snapshots() {
    let maps = IdMaps {
        source_space: "old".into(),
        target_space: "new".into(),
        assets: HashMap::new(),
        entities: HashMap::from([(
            "nodes",
            HashMap::from([("old-node".into(), "new-node".into())]),
        )]),
    };
    let revisions = HashMap::from([("new-node".into(), "node1:new".into())]);
    let detached = json!({"index":2,"state":"detached","nodeTitle":"Historical","sourceRevision":"node1:old","quote":"Snapshot"});
    let mut citations = json!([{"index":1,"state":"current","nodeId":"old-node","chunkId":"node:old-node","sourceRevision":"node1:old","locator":{"sourceRevision":"node1:old"}},detached]);
    remap_citations(&mut citations, &maps, &revisions).unwrap();
    assert_eq!(citations[0]["nodeId"], "new-node");
    assert_eq!(citations[0]["chunkId"], "node:new-node");
    assert_eq!(citations[0]["sourceRevision"], "node1:new");
    assert_eq!(citations[0]["locator"]["sourceRevision"], "node1:new");
    assert_eq!(citations[1], detached);
}
#[test]
fn rejects_stale_canonical_snapshot_before_rebinding() {
    let revision = json!("node1:current");
    let nodes = HashMap::from([("node", &revision)]);
    let citations = json!([{"state":"current","nodeId":"node","chunkId":"node:node","sourceRevision":"node1:stale","locator":{"sourceRevision":"node1:stale"}}]);
    assert!(validate_citations(&citations, &nodes)
        .unwrap_err()
        .contains("stale"));
}
#[test]
fn rejects_malformed_grounded_document_citations_before_remapping() {
    let value = json!({"nodes":[{"id":"node","data":{"knowledgeRevision":"node1:current"}}],"assistantMessages":[],"derivations":[{"groundedDocument":{"citations":[{"state":"current","nodeId":"node","chunkId":"node:node","nodeTitle":"Source","sourceRevision":"node1:current","quote":"Evidence","locator":"invalid"}]}}]});
    assert!(validate_source(&value)
        .unwrap_err()
        .contains("locator must be an object"));
}

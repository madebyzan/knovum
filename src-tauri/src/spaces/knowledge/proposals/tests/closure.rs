use serde_json::{json, Map};

use super::*;

fn claim(temp_id: &str) -> KnowledgeChange {
    KnowledgeChange::CreateObject {
        temp_id: temp_id.into(),
        object_type_id: "system:claim".into(),
        title: format!("Claim {temp_id}"),
        summary: None,
        status: "draft".into(),
        properties: Map::new(),
        position: None,
    }
}

fn supports(temp_id: &str, source: &str, target: &str) -> KnowledgeChange {
    KnowledgeChange::CreateRelation {
        temp_id: temp_id.into(),
        source_id: source.into(),
        target_id: target.into(),
        kind: "supports".into(),
        rationale: String::new(),
        confidence: None,
    }
}

fn provenance(target: &str) -> KnowledgeChange {
    KnowledgeChange::AddProvenance {
        target_kind: "node".into(),
        target_id: target.into(),
        source_node_id: "source-node".into(),
        locator: json!({}),
        source_revision: "rev".into(),
        quote: "quote".into(),
    }
}

fn changes() -> Vec<KnowledgeChange> {
    vec![
        claim("a"),
        supports("ra", "source-node", "a"),
        provenance("a"),
        claim("b"),
        supports("rb", "source-node", "b"),
        supports("rab", "a", "b"),
    ]
}

#[test]
fn operations_depend_on_the_temp_ids_they_name() {
    assert_eq!(
        dependencies(&changes()),
        vec![vec![], vec![0], vec![0], vec![], vec![3], vec![0, 3]]
    );
}

#[test]
fn deselecting_a_claim_drops_everything_that_needs_it() {
    assert_eq!(close(&changes(), &[0, 1, 2, 4, 5]), vec![0, 1, 2]);
    assert_eq!(
        close(&changes(), &[0, 1, 2, 3, 4, 5]),
        vec![0, 1, 2, 3, 4, 5]
    );
}

#[test]
fn a_selection_without_its_dependencies_keeps_nothing_dangling() {
    assert_eq!(close(&changes(), &[1, 2, 5]), Vec::<usize>::new());
    assert_eq!(close(&changes(), &[99]), Vec::<usize>::new());
}

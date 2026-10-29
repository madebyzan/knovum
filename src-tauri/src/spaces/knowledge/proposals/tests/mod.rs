use serde_json::{json, Map};
use sqlx::SqlitePool;

use super::*;
use crate::spaces::knowledge::grounding::evidence_span::EvidenceSpan;

async fn space() -> SqlitePool {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("sqlite");
    crate::spaces::db::migrate(&pool).await.expect("migrations");
    for statement in [
        "INSERT INTO spaces(id,kind,title,created_at,updated_at,last_opened_at) VALUES ('space','space','Space',1,1,1)",
        "INSERT INTO spaces_edit_state(space_id,revision,updated_at) VALUES ('space',0,1)",
        r#"INSERT INTO spaces_nodes(id,space_id,kind,initial_x,initial_y,data,created_at,updated_at) VALUES ('report','space','note',0,0,'{"kind":"note","title":"Q3 report","tiptapContent":{"type":"doc","content":[]},"plainText":"Revenue grew 12% in Q3."}',1,1)"#,
        r#"INSERT INTO spaces_knowledge_chunks(id,space_id,node_id,origin,kind,source_revision,text,locator,created_at,updated_at) VALUES ('report-chunk','space','report','node','node','rev-1','Revenue grew 12% in Q3.','{"sourceRevision":"rev-1","location":{"kind":"node"}}',1,1)"#,
    ] {
        sqlx::query(statement).execute(&pool).await.expect("fixture");
    }
    pool
}

fn span() -> EvidenceSpan {
    EvidenceSpan {
        chunk_id: "report-chunk".into(),
        node_id: "report".into(),
        source_revision: "rev-1".into(),
        locator: json!({"sourceRevision": "rev-1", "location": {"kind": "node"}}),
        quote_start: 0,
        quote_end: 16,
        quote: "Revenue grew 12%".into(),
    }
}

fn draft() -> ProposalDraft {
    ProposalDraft {
        origin: ProposalOrigin::Answer {
            conversation_id: "conversation".into(),
            message_id: "message".into(),
        },
        rationale: String::new(),
        changes: vec![
            KnowledgeChange::CreateObject {
                temp_id: "claim".into(),
                object_type_id: "system:claim".into(),
                title: "Revenue grew 12% in Q3.".into(),
                summary: None,
                status: "accepted".into(),
                properties: Map::from_iter([(
                    "statement".to_owned(),
                    json!("Revenue grew 12% in Q3."),
                )]),
                position: None,
            },
            KnowledgeChange::CreateRelation {
                temp_id: "supports".into(),
                source_id: "report".into(),
                target_id: "claim".into(),
                kind: "supports".into(),
                rationale: String::new(),
                confidence: None,
            },
            basis::provenance(&span(), "claim"),
        ],
        basis: vec![span()],
    }
}

const SYSTEM: Proposer<'static> = Proposer {
    kind: "system",
    id: None,
};

async fn propose_draft(pool: &SqlitePool) -> ProposalReceipt {
    let mut tx = pool.begin().await.unwrap();
    let receipt = propose(&mut tx, "space", &SYSTEM, draft(), 10)
        .await
        .unwrap();
    tx.commit().await.unwrap();
    receipt
}

async fn pending(pool: &SqlitePool) -> Vec<KnowledgeProposal> {
    list(&mut *pool.acquire().await.unwrap(), "space", None)
        .await
        .unwrap()
}

#[tokio::test]
async fn a_proposal_is_reviewed_with_its_evidence_and_dependencies() {
    let pool = space().await;
    let receipt = propose_draft(&pool).await;
    assert_eq!(receipt.state, ProposalState::Pending);
    let proposals = pending(&pool).await;
    assert_eq!(proposals.len(), 1);
    let proposal = &proposals[0];
    assert_eq!(proposal.freshness, ProposalFreshness::Current);
    assert_eq!(
        proposal
            .changes
            .iter()
            .map(|change| change.requires.clone())
            .collect::<Vec<_>>(),
        vec![vec![], vec![0], vec![0]]
    );
    assert_eq!(proposal.nodes.len(), 1);
    assert_eq!(
        (
            proposal.nodes[0].id.as_str(),
            proposal.nodes[0].title.as_str()
        ),
        ("report", "Q3 report")
    );
}

#[tokio::test]
async fn the_same_proposal_is_made_once_even_after_it_was_dismissed() {
    let pool = space().await;
    let first = propose_draft(&pool).await;
    let mut tx = pool.begin().await.unwrap();
    dismiss(&mut tx, &first.proposal_id, 11).await.unwrap();
    tx.commit().await.unwrap();
    let again = propose_draft(&pool).await;
    assert_eq!(again.proposal_id, first.proposal_id);
    assert_eq!(again.state, ProposalState::Dismissed);
    assert!(pending(&pool).await.is_empty());
}

#[tokio::test]
async fn accepting_commits_the_change_set_and_settles_the_proposal() {
    let pool = space().await;
    let receipt = propose_draft(&pool).await;
    let mut tx = pool.begin().await.unwrap();
    let acceptance = accept(&mut tx, &receipt.proposal_id, None, 12)
        .await
        .unwrap();
    tx.commit().await.unwrap();
    let claim = acceptance.resolved_ids["claim"]
        .as_str()
        .expect("created claim");
    let quote: String =
        sqlx::query_scalar("SELECT quote FROM spaces_provenance WHERE target_id=$1")
            .bind(claim)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(quote, "Revenue grew 12%");
    let relations: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM spaces_relations WHERE source_id='report' AND target_id=$1 AND kind='supports'",
    )
    .bind(claim)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(relations, 1);
    assert!(pending(&pool).await.is_empty());
    let mut tx = pool.begin().await.unwrap();
    assert!(accept(&mut tx, &receipt.proposal_id, None, 13)
        .await
        .is_err());
}

#[tokio::test]
async fn a_chosen_subset_keeps_only_what_can_stand_on_its_own() {
    let pool = space().await;
    let receipt = propose_draft(&pool).await;
    let mut tx = pool.begin().await.unwrap();
    let error = accept(&mut tx, &receipt.proposal_id, Some(&[1, 2]), 12)
        .await
        .unwrap_err();
    assert!(error.to_string().contains("knowledge-proposal-empty"));
    let acceptance = accept(&mut tx, &receipt.proposal_id, Some(&[0]), 12)
        .await
        .unwrap();
    tx.commit().await.unwrap();
    let provenance: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM spaces_provenance WHERE target_id=$1")
            .bind(acceptance.resolved_ids["claim"].as_str().unwrap())
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(provenance, 0);
}

#[tokio::test]
async fn a_proposal_whose_source_changed_cannot_be_accepted() {
    let pool = space().await;
    let receipt = propose_draft(&pool).await;
    sqlx::query(
        "UPDATE spaces_knowledge_chunks SET source_revision='rev-2' WHERE id='report-chunk'",
    )
    .execute(&pool)
    .await
    .unwrap();
    assert_eq!(
        pending(&pool).await[0].freshness,
        ProposalFreshness::BasisChanged
    );
    let mut tx = pool.begin().await.unwrap();
    let error = accept(&mut tx, &receipt.proposal_id, None, 12)
        .await
        .unwrap_err();
    assert!(error.to_string().contains("knowledge-proposal-outdated"));
}

#[tokio::test]
async fn a_proposal_about_a_deleted_node_is_outdated() {
    let pool = space().await;
    propose_draft(&pool).await;
    sqlx::query("UPDATE spaces_nodes SET deleted_at=20 WHERE id='report'")
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(
        pending(&pool).await[0].freshness,
        ProposalFreshness::TargetGone
    );
}

#[tokio::test]
async fn an_invalid_change_set_is_never_proposed() {
    let pool = space().await;
    let mut invalid = draft();
    invalid.changes.push(KnowledgeChange::CreateRelation {
        temp_id: "dangling".into(),
        source_id: "missing-node".into(),
        target_id: "claim".into(),
        kind: "supports".into(),
        rationale: String::new(),
        confidence: None,
    });
    let mut tx = pool.begin().await.unwrap();
    assert!(propose(&mut tx, "space", &SYSTEM, invalid, 10)
        .await
        .is_err());
    drop(tx);
    assert!(pending(&pool).await.is_empty());
}

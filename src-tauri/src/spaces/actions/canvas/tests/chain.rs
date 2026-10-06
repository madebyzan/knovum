use super::*;
use crate::spaces::{derivation::inputs, workflows::capture};

// Fixtures stand in for external inference; acceptance, provenance, capture and saving use production code.
async fn complete(
    pool: &SqlitePool,
    request: &StartSpaceActionRequest,
    output_id: &str,
    data: Value,
) -> (String, String) {
    let run = start(pool, request).await.unwrap();
    let revision = node(pool, output_id, data).await;
    let mut tx = db::begin_write(pool).await.unwrap();
    sqlx::query("INSERT INTO spaces_knowledge_derivations(id,space_id,output_node_id,action_kind,action_version,engine,status,created_at,updated_at,completed_at) VALUES($1,'space',$2,$3,1,'test-fixture','completed',1,2,2)")
        .bind(&run.id).bind(output_id).bind(&request.action_id).execute(&mut *tx).await.unwrap();
    let sources = request
        .sources
        .iter()
        .enumerate()
        .map(|(position, source)| inputs::DerivationInput {
            node_id: &source.node_id,
            role: if position == 0 { "primary" } else { "source" },
            source_revision: &source.knowledge_revision,
            revision_basis: inputs::RevisionBasis::Knowledge,
        })
        .collect::<Vec<_>>();
    inputs::insert(&mut tx, &run.id, &sources).await.unwrap();
    sqlx::query("UPDATE spaces_runs SET state='succeeded',completed_at=2 WHERE id=$1")
        .bind(&run.id)
        .execute(&mut *tx)
        .await
        .unwrap();
    tx.commit().await.unwrap();
    (run.id, revision)
}
#[tokio::test]
async fn completed_branching_derivation_chain_captures_one_replaceable_note_input() {
    let pool = database().await;
    let note = node(&pool,"note",json!({"kind":"note","title":"Idea","plainText":"A campaign","tiptapContent":{"type":"doc","content":[]}})).await;
    let target = Some(TaskTargetRef::Agent {
        runtime_kind: crate::ai::contracts::ExternalAgentKind::Codex,
    });
    let image_parameters =
        json!({"title":"Image","intent":"Illustrate","style":"Flat","aspectRatio":"1:1"});
    let (_,image1) = complete(&pool,&request("generate-visualization",&[("note",&note)],image_parameters.clone(),target.clone()),"image1",json!({"kind":"image","title":"One","sourceRevision":"asset:1","assetPath":"/owned/one.png"})).await;
    let (_,image2) = complete(&pool,&request("generate-visualization",&[("note",&note)],image_parameters,target.clone()),"image2",json!({"kind":"image","title":"Two","sourceRevision":"asset:2","assetPath":"/owned/two.png"})).await;
    let (_,video) = complete(&pool,&request("generate-video",&[("image1",&image1),("image2",&image2)],json!({"title":"Video","prompt":"Animate","aspectRatio":"16:9","durationSeconds":5}),target.clone()),"video",json!({"kind":"video","title":"Video","sourceRevision":"asset:video","assetPath":"/owned/video.mp4"})).await;
    complete(
        &pool,
        &request(
            "draft-document",
            &[("image1", &image1), ("image2", &image2), ("video", &video)],
            json!({"title":"Report","purpose":"Describe the campaign"}),
            target,
        ),
        "document",
        json!({"kind":"markdown","title":"Report","text":"Campaign report"}),
    )
    .await;
    let preview = capture::preview(&pool, "space", vec!["document".into()])
        .await
        .unwrap();
    assert!(
        preview.issues.is_empty(),
        "{}",
        serde_json::to_string(&preview.issues).unwrap()
    );
    let candidate = preview.candidate.unwrap();
    assert_eq!(candidate.steps.len(), 4);
    assert_eq!(candidate.input_slots.len(), 1);
    assert_eq!(candidate.input_slots[0].kind, "note");
    let video = candidate
        .steps
        .iter()
        .find(|step| step.action_id == "generate-video")
        .unwrap();
    assert_eq!(video.inputs.len(), 2);
    assert!(video.inputs.iter().all(|input| matches!(
        input.source,
        capture::contracts::CandidateSource::StepOutput { .. }
    )));
    let saved = capture::save(
        &pool,
        capture::SaveCapturedWorkflowRequest {
            request_id: uuid::Uuid::new_v4().to_string(),
            space_id: "space".into(),
            selected_node_ids: vec!["document".into()],
            source_digest: preview.source_digest.unwrap(),
            name: "Campaign".into(),
            output_names: candidate
                .output_slots
                .iter()
                .map(|output| (output.id.clone(), "Report".into()))
                .collect(),
        },
    )
    .await
    .unwrap();
    assert_eq!(saved.candidate.steps.len(), 4);
    assert_eq!(capture::list(&pool).await.unwrap().items.len(), 1);
    let serialized = serde_json::to_value(&saved.candidate).unwrap().to_string();
    assert!(!serialized.contains("connectionId"));
    assert!(!serialized.contains("/owned/"));
    assert!(saved.candidate.steps.iter().all(|step| matches!(
        step.execution_spec.target,
        Some(crate::ai::tasks::TaskTargetSelection::Agent { .. })
    )));
}
#[tokio::test]
async fn local_document_export_is_capturable_without_model_configuration() {
    let pool = database().await;
    let revision = node(
        &pool,
        "source",
        json!({"kind":"markdown","title":"Report","text":"Research"}),
    )
    .await;
    complete(&pool,&request("export-document-as-docx",&[("source",&revision)],json!({}),None),"output",json!({"kind":"file","title":"Report.docx","sourceRevision":"asset:docx","assetPath":"/owned/report.docx"})).await;
    let preview = capture::preview(&pool, "space", vec!["output".into()])
        .await
        .unwrap();
    let candidate = preview.candidate.unwrap();
    assert_eq!(candidate.steps[0].action_id, "export-document-as-docx");
    assert!(candidate.steps[0].execution_spec.target.is_none());
}

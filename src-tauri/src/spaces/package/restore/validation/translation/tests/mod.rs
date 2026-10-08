use serde_json::{json, Value};

fn request() -> super::RestorePackageRequest {
    let source =
        json!({"sourceSegmentId":"base-segment","sourceSegmentRevision":1,"text":"Original"});
    let output = json!({
        "stepId":"step","sourceSegmentId":"base-segment","sourceSegmentRevision":1,
        "sourceText":"Original","translatedText":"Model translation"
    });
    let mut request = json!({
        "space":{"id":"space","kind":"space","title":"Space","lastOpenedAt":1,"createdAt":1,"updatedAt":4},
        "structureNodes":[],"layouts":[],"relationStyles":[],"objectTypes":[],"nodes":[],
        "scenes":[],"relations":[],"derivations":[],"runEvents":[],
        "externalAgentConversations":[],"externalAgentTurns":[],"conversations":[],
        "conversationMessages":[],"voiceProfiles":[],"voiceRevisions":[],"voiceCandidates":[],
        "speechProjects":[],"speechRenders":[],"speechSourceSeparations":[],"provenance":[],
        "knowledgeEvents":[],"chunks":[],"runs":[],"runSteps":[],"runAttempts":[],
        "speechProjectRevisions":[],"speechSegments":[]
    });
    request["runs"] = json!([run(source.clone())]);
    request["runSteps"] = json!([step(source, output.clone())]);
    request["runAttempts"] = json!([attempt(output)]);
    request["speechProjectRevisions"] = json!([
        revision("base", 1, Value::Null),
        revision("edited", 2, provenance())
    ]);
    request["speechSegments"] = json!([
        segment(
            "base-segment",
            "base",
            "Original",
            Value::Null,
            Value::Null,
            Value::Null
        ),
        segment(
            "edited-segment",
            "edited",
            "Human edited translation",
            json!("base-segment"),
            json!(1),
            json!("Original")
        )
    ]);
    serde_json::from_value(request).unwrap()
}

fn run(source: Value) -> Value {
    json!({
        "id":"run","parentRunId":null,"retryOfRunId":null,"forkOfRunId":null,
        "principalKind":"user","principalId":"local-user","kind":"workflow","state":"succeeded",
        "stateVersion":3,"policySnapshot":{},"inputSnapshot":{
            "workflow":super::WORKFLOW,"projectId":"project","revisionId":"base",
            "expectedRevision":1,"inputRevision":"sha256:input","targetLanguage":"zh-CN",
            "localOnly":false,"provider":"openai","model":"model","baseUrl":"https://example.test/v1",
            "executionTarget":target(),"segments":[source]},
        "budget":{},"inputTokens":0,"outputTokens":0,"totalTokens":0,"stepAttempts":1,
        "expandedSteps":1,"outputBytes":0,"createdAt":1,"updatedAt":3,"completedAt":3
    })
}

fn step(source: Value, output: Value) -> Value {
    json!({
        "id":"step","runId":"run","definitionNodeId":"translation:0","instancePath":"translation:0",
        "nodeKind":super::NODE_KIND,"dependsOn":[],"configuration":{
            "projectId":"project","revisionId":"base","expectedRevision":1,
            "inputRevision":"sha256:input","targetLanguage":"zh-CN","localOnly":false,
            "provider":"openai","model":"model","baseUrl":"https://example.test/v1",
            "executionTarget":target(),"ordinal":0,"segment":source},
        "state":"succeeded","stateVersion":2,"resolvedInput":{},"output":output,
        "createdAt":1,"updatedAt":3
    })
}

fn attempt(output: Value) -> Value {
    json!({
        "id":"attempt","runId":"run","stepInstanceId":"step","attemptNumber":1,
        "state":"succeeded","requestDigest":"a".repeat(64),"idempotencyKey":"step",
        "externalTaskId":null,"checkpoint":null,"output":output,"errorCode":null,
        "startedAt":2,"finishedAt":3
    })
}

fn target() -> Value {
    json!({"type":"model","connectionId":"connection","modelId":"model","temperature":0.0})
}

fn provenance() -> Value {
    json!({"runId":"run","targetLanguage":"zh-CN","provider":"openai","model":"model","localOnly":false})
}

fn revision(id: &str, revision: i64, translation: Value) -> Value {
    json!({
        "id":id,"projectId":"project","revision":revision,"sourceRevision":"sha256:source",
        "pronunciationLexicon":{},"outputConfig":{"translation":translation},
        "outputAssetPath":null,"outputSha256":null,"outputSampleRate":null,
        "outputSampleCount":null,"outputPublishedAt":null,"createdAt":revision
    })
}

fn segment(
    id: &str,
    revision_id: &str,
    text: &str,
    source_id: Value,
    source_revision: Value,
    source_text: Value,
) -> Value {
    json!({
        "id":id,"projectRevisionId":revision_id,"ordinal":0,"chapterKey":null,"speakerKey":null,
        "text":text,"normalizedText":text,"voiceRevisionId":"voice","language":"zh-CN",
        "targetStartMs":0,"targetEndMs":1000,"sourceSegmentId":source_id,
        "sourceSegmentRevision":source_revision,"sourceText":source_text,"state":"ready",
        "stateReason":null,"createdAt":1,"updatedAt":1
    })
}

#[test]
fn accepts_base_to_user_edited_successor() {
    super::validate(&request()).unwrap();
}

#[test]
fn rejects_run_with_forged_base_source_identity() {
    let mut request = request();
    request.runs[0].input_snapshot["segments"][0]["text"] = json!("Forged");
    request.run_steps[0].configuration["segment"]["text"] = json!("Forged");
    request.run_steps[0].output.as_mut().unwrap()["sourceText"] = json!("Forged");
    request.run_attempts[0].output = request.run_steps[0].output.clone();
    assert!(super::validate(&request).is_err());
}

#[test]
fn rejects_succeeded_translation_step_without_attempt() {
    let mut request = request();
    request.run_attempts.clear();
    request.runs[0].step_attempts = 0;
    assert!(super::validate(&request).is_err());
}

use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use crate::spaces::package::ImportedPackageAsset;

const COLLECTIONS: &[&str] = &[
    "nodes",
    "structureNodes",
    "layouts",
    "relationStyles",
    "scenes",
    "relations",
    "derivations",
    "runs",
    "runSteps",
    "runAttempts",
    "runEvents",
    "externalAgentConversations",
    "externalAgentTurns",
    "conversations",
    "conversationMessages",
    "objectTypes",
    "provenance",
    "chunks",
    "knowledgeEvents",
    "voiceProfiles",
    "voiceRevisions",
    "voiceCandidates",
    "speechProjects",
    "speechProjectRevisions",
    "speechSegments",
    "speechRenders",
    "speechSourceSeparations",
];

pub(super) fn package(selected: bool) -> (Value, Vec<ImportedPackageAsset>) {
    let source_kind = if selected { "reference" } else { "designed" };
    let source_revision = format!("sha256:{}", "1".repeat(64));
    let reference_sha = "2".repeat(64);
    let preview_sha = "3".repeat(64);
    let source = if selected {
        json!({"kind":"reference","sourcePath":null,"sourceRevision":source_revision.clone(),"startMs":0,"endMs":10000,"referenceText":"sample","referenceTextConfirmed":true})
    } else {
        json!({"kind":"designed","description":"calm narrator"})
    };
    let frozen =
        selected.then(|| json!({"byteLength":32000,"sourceRevision":source_revision.clone()}));
    let output = output(
        source_kind,
        (!selected).then_some("calm narrator"),
        &preview_sha,
    );
    let mut manifest = base();
    manifest["runs"] = json!([run(frozen.clone())]);
    manifest["runSteps"] = json!([step(source, frozen, output.clone())]);
    manifest["runAttempts"] = json!([attempt(output)]);
    manifest["voiceCandidates"] = json!([candidate(
        selected,
        &source_revision,
        &reference_sha,
        &preview_sha
    )]);
    if selected {
        manifest["voiceProfiles"] = json!([{
            "id":"profile","spaceId":"original","name":"Voice","sourceKind":"reference",
            "activeRevisionId":"revision","archivedAt":null,"createdAt":2,"updatedAt":4
        }]);
        manifest["voiceRevisions"] = json!([{
            "id":"revision","profileId":"profile","revision":1,"engineId":"voxcpm2",
            "modelSpecId":crate::spaces::speech::voice_candidates::MODEL_SPEC_ID,
            "language":"zh-CN","presetVoiceId":null,"referenceAssetId":"knovum-package-asset:voice",
            "referenceSourceRevision":format!("sha256:{reference_sha}"),"referenceText":"sample",
            "designDescription":null,"synthesisConfig":{"speed":1.0,"sampleRate":48000,"seed":0},
            "authorization":{"declared":true,"declaredAt":4,"referenceTranscriptConfirmed":true,
                "candidateId":"run","candidateRunId":"run"},"createdAt":4
        }]);
    }
    let assets = ["reference", "preview", "voice"]
        .into_iter()
        .map(|token| ImportedPackageAsset {
            token: token.into(),
            path: format!("/managed/{token}.wav"),
        })
        .collect();
    (manifest, assets)
}

fn base() -> Value {
    let mut value = json!({
        "format":"knovum-space","version":super::super::super::super::super::PACKAGE_VERSION,
        "exportedAt":5,"space":{"id":"original","kind":"space","title":"Imported",
            "archivedAt":null,"lastOpenedAt":1,"createdAt":1,"updatedAt":5}
    });
    for collection in COLLECTIONS {
        value[*collection] = json!([]);
    }
    value
}

fn run(frozen: Option<Value>) -> Value {
    json!({
        "id":"run","parentRunId":null,"retryOfRunId":null,"forkOfRunId":null,
        "principalKind":"user","principalId":"local-user","kind":"workflow","state":"succeeded",
        "stateVersion":3,"policySnapshot":{},"inputSnapshot":{"workflow":"speech_voice_candidate_v1",
            "requestDigest":"a".repeat(64),"referenceSource":frozen},
        "budget":{},"inputTokens":0,"outputTokens":0,"totalTokens":0,"stepAttempts":1,
        "expandedSteps":1,"outputBytes":0,"createdAt":1,"updatedAt":3,"completedAt":3
    })
}

fn step(source: Value, frozen: Option<Value>, output: Value) -> Value {
    json!({
        "id":"step","runId":"run","definitionNodeId":"candidate","instancePath":"candidate",
        "nodeKind":"speech_voice_candidate","dependsOn":[],"configuration":{"request":{
            "requestId":"run","spaceId":"original","language":"zh-CN","text":"sample",
            "source":source,"authorized":true,"seed":0},"referenceSource":frozen},
        "state":"succeeded","stateVersion":2,"resolvedInput":{},"output":output,
        "createdAt":1,"updatedAt":3
    })
}

fn attempt(output: Value) -> Value {
    json!({
        "id":"attempt","runId":"run","stepInstanceId":"step","attemptNumber":1,
        "state":"succeeded","requestDigest":hex::encode(Sha256::digest(b"{}")),
        "idempotencyKey":"step","externalTaskId":null,"checkpoint":null,"result":output,
        "errorCode":null,"startedAt":2,"finishedAt":3
    })
}

fn candidate(
    selected: bool,
    source_revision: &str,
    reference_sha: &str,
    preview_sha: &str,
) -> Value {
    json!({
        "id":"run","runId":"run","spaceId":"original",
        "sourceKind":if selected {"reference"} else {"designed"},"language":"zh-CN",
        "sampleText":"sample","designDescription":if selected {Value::Null} else {json!("calm narrator")},
        "referenceText":"sample","inputSourceRevision":if selected {json!(source_revision)} else {Value::Null},
        "inputByteLength":if selected {json!(32000)} else {Value::Null},
        "referenceAssetPath":"knovum-package-asset:reference","referenceSha256":reference_sha,
        "referenceByteLength":32044,"previewAssetPath":"knovum-package-asset:preview",
        "previewSha256":preview_sha,"previewSampleRate":48000,"previewSampleCount":48000,
        "modelSpecId":crate::spaces::speech::voice_candidates::MODEL_SPEC_ID,"seed":0,
        "synthesisConfig":{"speed":1.0,"sampleRate":48000,"seed":0},
        "authorization":{"declared":true,"declaredAt":3,"referenceTranscriptConfirmed":true},
        "status":if selected {"selected"} else {"ready"},
        "auditionedAt":if selected {json!(4)} else {Value::Null},
        "selectedVoiceProfileId":if selected {json!("profile")} else {Value::Null},
        "selectedVoiceRevisionId":if selected {json!("revision")} else {Value::Null},
        "selectedAt":if selected {json!(4)} else {Value::Null},"rejectedAt":null,"createdAt":3
    })
}

fn output(source_kind: &str, description: Option<&str>, preview_sha: &str) -> Value {
    json!({
        "candidateId":"run","runId":"run","spaceId":"original","sourceKind":source_kind,"seed":0,
        "language":"zh-CN","sampleText":"sample","designDescription":description,
        "referenceText":"sample","audio":{"sha256":preview_sha,"sampleRate":48000,
            "sampleCount":48000},"status":"ready","auditionedAt":null,
        "selectedVoiceProfileId":null,"selectedVoiceRevisionId":null,"selectedAt":null,
        "rejectedAt":null,"createdAt":3
    })
}

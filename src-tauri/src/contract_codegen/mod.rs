use std::path::Path;

use schemars::schema_for;
use ts_rs::Config;

// The fields exist only to collect one closed schema graph for Python generation.
#[allow(dead_code)]
#[derive(schemars::JsonSchema)]
struct InferenceWireSchemas {
    initialize_params: crate::ai::contracts::runtime::RuntimeInitializeParams,
    initialize_result: crate::ai::contracts::runtime::RuntimeInitializeResult,
    health_params: crate::ai::contracts::runtime::RuntimeHealthParams,
    health_result: crate::ai::contracts::runtime::RuntimeHealthResult,
    cancel_params: crate::ai::contracts::runtime::RuntimeCancelParams,
    cancel_result: crate::ai::contracts::runtime::RuntimeCancelResult,
    ack_chunk_params: crate::ai::contracts::runtime::RuntimeAckChunkParams,
    shutdown_params: crate::ai::contracts::runtime::RuntimeShutdownParams,
    shutdown_result: crate::ai::contracts::runtime::RuntimeShutdownResult,
    embedding_params: crate::ai::contracts::embedding_wire::EmbeddingEncodeParams,
    embedding_result: crate::ai::contracts::embedding_wire::EmbeddingEncodeResult,
    image_segment_params: crate::ai::contracts::image_wire::ImageSegmentParams,
    image_segment_result: crate::ai::contracts::image_wire::ImageSegmentResult,
    speech_transcribe_params: crate::ai::contracts::speech_wire::SpeechTranscribeParams,
    speech_transcribe_result: crate::ai::contracts::speech_wire::SpeechTranscribeResult,
    speech_transcript_artifact: crate::ai::contracts::speech_wire::SpeechTranscriptArtifact,
    speech_progress_event: crate::ai::contracts::speech_wire::SpeechProgressEvent,
    speech_progress_event_type: crate::ai::contracts::speech_wire::SpeechProgressEventType,
    speech_align_params: crate::ai::contracts::speech_wire::SpeechAlignParams,
    speech_align_result: crate::ai::contracts::speech_wire::SpeechAlignResult,
    speech_diarize_params: crate::ai::contracts::speech_wire::SpeechDiarizeParams,
    speech_diarize_result: crate::ai::contracts::speech_wire::SpeechDiarizeResult,
    speech_synthesize_params: crate::ai::contracts::speech_wire::SpeechSynthesizeParams,
    speech_synthesis_chunk_event: crate::ai::contracts::speech_wire::SpeechSynthesisChunkEvent,
    speech_synthesis_chunk_granularity:
        crate::ai::contracts::speech_wire::SpeechSynthesisChunkGranularity,
    speech_synthesize_result: crate::ai::contracts::speech_wire::SpeechSynthesizeResult,
    speech_separate_params: crate::ai::contracts::speech_wire::SpeechSeparateParams,
    speech_separate_result: crate::ai::contracts::speech_wire::SpeechSeparateResult,
}

mod types;

pub fn export(project_root: &Path) -> Result<(), String> {
    let output = project_root.join("src/shared/contracts/generated");
    let config = Config::new().with_out_dir(&output).with_large_int("number");
    types::export(&config)?;
    export_runtime_values(&output)?;
    let wire_schema = project_root.join("python/schemas/runtime.schema.json");
    std::fs::create_dir_all(
        wire_schema
            .parent()
            .ok_or("Inference schema has no parent")?,
    )
    .map_err(io_error)?;
    write_json(&wire_schema, &schema_for!(InferenceWireSchemas))?;
    let spaces_output = project_root.join("src/features/spaces/contracts/generated");
    let relation_kinds = schema_for!(crate::spaces::knowledge::contracts::RelationKind);
    let relation_statuses = schema_for!(crate::spaces::knowledge::contracts::RelationStatus);
    let source = format!(
        concat!(
            "// Generated from Rust contracts. Do not edit.\n",
            "export const maxKnowledgePassageBytes = {} as const;\n",
            "export const packageAssetPrefix = {:?} as const;\n",
            "export const relationKinds = {} as const;\n",
            "export const relationStatuses = {} as const;\n",
        ),
        crate::spaces::knowledge::indexing::MAX_KNOWLEDGE_PASSAGE_BYTES,
        crate::spaces::assets::PACKAGE_ASSET_PREFIX,
        relation_kinds
            .get("enum")
            .ok_or("Relation kinds must be an enum")?,
        relation_statuses
            .get("enum")
            .ok_or("Relation statuses must be an enum")?,
    );
    std::fs::write(spaces_output.join("contract-values.ts"), source).map_err(io_error)?;
    export_schemas(&spaces_output)
}

fn export_schemas(output: &Path) -> Result<(), String> {
    std::fs::create_dir_all(output).map_err(io_error)?;
    let action_schemas = crate::spaces::actions::definitions()
        .iter()
        .map(|definition| {
            (
                definition.id.to_owned(),
                crate::spaces::actions::parameter_schema(definition),
            )
        })
        .collect::<serde_json::Map<_, _>>();
    write_json(
        &output.join("space-action-parameters.schema.json"),
        &action_schemas,
    )
}

fn write_json(path: &Path, value: &impl serde::Serialize) -> Result<(), String> {
    let bytes = serde_json::to_vec_pretty(value).map_err(|error| error.to_string())?;
    std::fs::write(path, bytes).map_err(io_error)
}

fn io_error(error: std::io::Error) -> String {
    error.to_string()
}

#[cfg(test)]
mod tests;

fn export_runtime_values(output: &Path) -> Result<(), String> {
    let enums = [
        (
            "usagePurposes",
            schema_for!(crate::ai::usage::contracts::UsagePurpose),
        ),
        (
            "providerKinds",
            schema_for!(crate::ai::contracts::ProviderKind),
        ),
        (
            "executionCapabilities",
            schema_for!(crate::ai::contracts::ExecutionCapability),
        ),
        (
            "routingPurposes",
            schema_for!(crate::ai::contracts::RoutingPurpose),
        ),
        ("agentKinds", schema_for!(crate::ai::contracts::AgentKind)),
    ];
    let mut source = String::from("// Generated from Rust contracts. Do not edit.\n");
    for (name, schema) in enums {
        let values = schema
            .get("enum")
            .ok_or("Expected an enum schema for runtime values")?;
        source.push_str(&format!(
            "export const {name} = {} as const;\n",
            serde_json::to_string(values).map_err(|error| error.to_string())?
        ));
    }
    let purposes = schema_for!(crate::ai::contracts::RoutingPurpose);
    let mut policies = serde_json::Map::new();
    for value in purposes
        .get("enum")
        .and_then(serde_json::Value::as_array)
        .ok_or("Routing purposes must be an enum")?
    {
        let purpose: crate::ai::contracts::RoutingPurpose =
            serde_json::from_value(value.clone()).map_err(|e| e.to_string())?;
        policies.insert(
            value
                .as_str()
                .ok_or("Routing purpose must be a string")?
                .to_owned(),
            serde_json::to_value(purpose.execution_capability()).map_err(|e| e.to_string())?,
        );
    }
    source.push_str(&format!(
        "export const routingCapabilities = {} as const;\n",
        serde_json::to_string(&policies).map_err(|e| e.to_string())?
    ));
    std::fs::write(output.join("contract-values.ts"), source).map_err(io_error)
}

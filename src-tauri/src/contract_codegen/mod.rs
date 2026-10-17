use std::path::Path;

use schemars::{schema_for, Schema};
use ts_rs::Config;

// Collect the neutral native worker protocol into a closed schema graph.
#[allow(dead_code)]
#[derive(schemars::JsonSchema)]
struct InferenceWireSchemas {
    diagnostic: crate::ai::contracts::diagnostics::RuntimeDiagnostic,
    initialize_params: crate::ai::contracts::runtime::RuntimeInitializeParams,
    initialize_result: crate::ai::contracts::runtime::RuntimeSession,
    health_params: crate::ai::contracts::runtime::RuntimeHealthParams,
    health_result: crate::ai::contracts::runtime::RuntimeHealthObservation,
    cancel_params: crate::ai::contracts::runtime::RuntimeCancelParams,
    cancel_result: crate::ai::contracts::runtime::RuntimeCancellationAcknowledgement,
    ack_chunk_params: crate::ai::contracts::runtime::RuntimeAckChunkParams,
    shutdown_params: crate::ai::contracts::runtime::RuntimeShutdownParams,
    shutdown_result: crate::ai::contracts::runtime::RuntimeShutdownAcknowledgement,
    embedding_params: crate::ai::contracts::embedding_wire::EmbeddingEncodeParams,
    embedding_result: crate::ai::contracts::embedding_wire::EmbeddingEncoding,
    image_compute_params: crate::ai::contracts::image_compute::ImageComputeParams,
    image_compute_output: crate::ai::contracts::image_compute::ImageComputationOutput,
    image_segment_params: crate::ai::contracts::image_wire::ImageSegmentParams,
    image_segment_result: crate::ai::contracts::image_wire::ImageSegmentation,
    speech_transcribe_params: crate::ai::contracts::speech_wire::SpeechTranscribeParams,
    speech_transcribe_result: crate::ai::contracts::speech_wire::SpeechTranscription,
    speech_transcript_artifact: crate::ai::contracts::speech_wire::SpeechTranscriptArtifact,
    speech_progress_event: crate::ai::contracts::speech_wire::SpeechProgressEvent,
    speech_progress_event_type: crate::ai::contracts::speech_wire::SpeechProgressEventType,
    speech_align_params: crate::ai::contracts::speech_wire::SpeechAlignParams,
    speech_align_result: crate::ai::contracts::speech_wire::SpeechAlignment,
    speech_diarize_params: crate::ai::contracts::speech_wire::SpeechDiarizeParams,
    speech_diarize_result: crate::ai::contracts::speech_wire::SpeechDiarization,
    speech_synthesize_params: crate::ai::contracts::speech_wire::SpeechSynthesizeParams,
    speech_synthesis_chunk_event: crate::ai::contracts::speech_wire::SpeechSynthesisChunkEvent,
    speech_synthesis_chunk_granularity:
        crate::ai::contracts::speech_wire::SpeechSynthesisChunkGranularity,
    speech_synthesize_result: crate::ai::contracts::speech_wire::SpeechSynthesis,
    speech_separate_params: crate::ai::contracts::speech_wire::SpeechSeparateParams,
    speech_separate_result: crate::ai::contracts::speech_wire::SpeechSeparation,
}

mod action_schemas;
mod bindings;

pub fn export(project_root: &Path) -> Result<(), String> {
    let output = project_root.join("src/shared/contracts/generated");
    let config = Config::new().with_out_dir(&output).with_large_int("number");
    bindings::export(&config)?;
    export_runtime_values(&output)?;
    let settings_values = enum_values_source(&[
        (
            "maintenanceOperations",
            schema_for!(crate::backup::contracts::MaintenanceOperation),
        ),
        (
            "maintenanceStatuses",
            schema_for!(crate::backup::contracts::MaintenanceStatus),
        ),
    ])?;
    std::fs::write(
        project_root.join("src/features/settings/contracts/generated/contract-values.ts"),
        settings_values,
    )
    .map_err(io_error)?;
    let wire_schema =
        project_root.join("src-tauri/crates/inference-contracts/inference.schema.json");
    std::fs::create_dir_all(
        wire_schema
            .parent()
            .ok_or("Inference schema has no parent")?,
    )
    .map_err(io_error)?;
    write_json(&wire_schema, &schema_for!(InferenceWireSchemas))?;
    write_json(
        &project_root.join("python/schemas/agent_runtime.schema.json"),
        &schema_for!(crate::agent_runtime::protocol::AgentWireSchemas),
    )?;
    let agent_values = format!(
        "# Generated from Rust contracts. Do not edit.\nPROTOCOL_VERSION = {}\nGRAPH_VERSION = {:?}\n",
        crate::agent_runtime::protocol::PROTOCOL_VERSION,
        crate::agent_runtime::protocol::GRAPH_VERSION,
    );
    let agent_contracts = project_root.join("python/src/knovum_agent_runtime/contracts");
    std::fs::create_dir_all(&agent_contracts).map_err(io_error)?;
    std::fs::write(agent_contracts.join("values.py"), agent_values).map_err(io_error)?;
    let spaces_output = project_root.join("src/features/spaces/contracts/generated");
    let relation_kinds = schema_for!(crate::spaces::knowledge::contracts::RelationKind);
    let relation_statuses = schema_for!(crate::spaces::knowledge::contracts::RelationStatus);
    let mut source = format!(
        concat!(
            "// Generated from Rust contracts. Do not edit.\n",
            "export const maxKnowledgePassageBytes = {} as const;\n",
            "export const relationKinds = {} as const;\n",
            "export const relationStatuses = {} as const;\n",
            "export const visualKnowledgeLimits = {} as const;\n",
        ),
        crate::spaces::knowledge::indexing::MAX_KNOWLEDGE_PASSAGE_BYTES,
        relation_kinds
            .get("enum")
            .ok_or("Relation kinds must be an enum")?,
        relation_statuses
            .get("enum")
            .ok_or("Relation statuses must be an enum")?,
        serde_json::json!({
            "description": crate::spaces::knowledge::validation::nodes::visual::MAX_DESCRIPTION,
            "detectedText": crate::spaces::knowledge::validation::nodes::visual::MAX_DETECTED_TEXT,
            "tags": crate::spaces::knowledge::validation::nodes::visual::MAX_TAGS,
            "tag": crate::spaces::knowledge::validation::nodes::visual::MAX_TAG,
        }),
    );
    std::fs::write(spaces_output.join("contract-values.ts"), source).map_err(io_error)?;
    export_schemas(&spaces_output)
}

fn export_schemas(output: &Path) -> Result<(), String> {
    std::fs::create_dir_all(output).map_err(io_error)?;
    let action_schemas = action_schemas::export()?;
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
            "chatModelProtocols",
            schema_for!(crate::ai::contracts::ChatModelProtocol),
        ),
        (
            "externalAgentKinds",
            schema_for!(crate::ai::contracts::ExternalAgentKind),
        ),
        (
            "preferenceThemes",
            schema_for!(crate::preferences::ThemePreference),
        ),
        (
            "preferenceDensities",
            schema_for!(crate::preferences::DensityPreference),
        ),
        (
            "preferenceTextSizes",
            schema_for!(crate::preferences::TextSizePreference),
        ),
        (
            "preferenceLocales",
            schema_for!(crate::preferences::LocalePreference),
        ),
    ];
    let mut source = enum_values_source(&enums)?;
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
    source.push_str(&format!(
        concat!(
            "export const preferencesSchemaVersion = {} as const;\n",
            "export const workbenchWidthLimits = {} as const;\n",
            "export const preferenceDefaults = {} as const;\n",
        ),
        crate::preferences::SCHEMA_VERSION,
        serde_json::json!({
            "sidebar": [crate::preferences::SIDEBAR_WIDTH_MIN, crate::preferences::SIDEBAR_WIDTH_MAX],
            "rightSidebar": [crate::preferences::RIGHT_SIDEBAR_WIDTH_MIN, crate::preferences::RIGHT_SIDEBAR_WIDTH_MAX],
        }),
        serde_json::to_string(&crate::preferences::Preferences::default())
            .map_err(|error| error.to_string())?,
    ));
    std::fs::write(output.join("contract-values.ts"), source).map_err(io_error)
}

fn enum_values_source(enums: &[(&str, Schema)]) -> Result<String, String> {
    Ok(format!(
        "// Generated from Rust contracts. Do not edit.\n{}",
        enum_values(enums)?
    ))
}

fn enum_values(enums: &[(&str, Schema)]) -> Result<String, String> {
    let mut source = String::new();
    for (name, schema) in enums {
        let values = schema
            .get("enum")
            .ok_or("Expected an enum schema for runtime values")?;
        source.push_str(&format!(
            "export const {name} = {} as const;\n",
            serde_json::to_string(values).map_err(|error| error.to_string())?
        ));
    }
    Ok(source)
}

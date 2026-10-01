use std::path::Path;

use schemars::schema_for;
use ts_rs::Config;

mod types;

pub fn export(project_root: &Path) -> Result<(), String> {
    let output = project_root.join("src/shared/contracts/generated");
    let config = Config::new().with_out_dir(&output).with_large_int("number");
    types::export(&config)?;
    export_runtime_values(&output)?;
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

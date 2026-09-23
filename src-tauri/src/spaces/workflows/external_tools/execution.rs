use serde_json::Value;
use sqlx::SqlitePool;

use super::{client, fingerprint, repository, ExternalToolBinding, ExternalToolEffect};
use crate::spaces::runs::{AttemptClaim, StepOutcome};

const MAX_RESULT_BYTES: usize = 1_048_576;

pub(crate) async fn execute(
    pool: &SqlitePool,
    claim: &AttemptClaim,
    binding_id: &str,
    expected_fingerprint: &str,
) -> Result<StepOutcome, String> {
    let binding = repository::load(pool, binding_id).await?;
    if claim
        .policy_snapshot
        .get("testMode")
        .and_then(Value::as_bool)
        == Some(true)
        && !matches!(binding.effect, ExternalToolEffect::ReadOnly)
    {
        return Ok(StepOutcome::Completed {
            output: serde_json::json!({
                "isolated": true, "simulatedExternalTool": binding_id,
                "input": claim.resolved_input,
            }),
            selected_branch: None,
        });
    }
    let grant_id = claim
        .policy_snapshot
        .get("grantId")
        .and_then(Value::as_str)
        .ok_or("permission-denied: Run does not contain an authorization grant")?;
    let output = execute_binding(
        pool,
        &claim.home_space_id,
        grant_id,
        binding_id,
        expected_fingerprint,
        &claim.resolved_input,
    )
    .await?;
    Ok(StepOutcome::Completed {
        output,
        selected_branch: None,
    })
}

pub(crate) async fn execute_direct(
    pool: &SqlitePool,
    space_id: &str,
    grant_id: &str,
    binding_id: &str,
    expected_fingerprint: &str,
    input: &Value,
) -> Result<Value, String> {
    execute_binding(
        pool,
        space_id,
        grant_id,
        binding_id,
        expected_fingerprint,
        input,
    )
    .await
}

async fn execute_binding(
    pool: &SqlitePool,
    space_id: &str,
    grant_id: &str,
    binding_id: &str,
    expected_fingerprint: &str,
    input: &Value,
) -> Result<Value, String> {
    let binding = repository::load(pool, binding_id).await?;
    if !binding.enabled || binding.space_id != space_id {
        return Err(
            "external-tool-unavailable: binding is disabled or belongs to another Space".into(),
        );
    }
    if binding.schema_fingerprint != expected_fingerprint {
        return Err(
            "external-tool-schema-changed: Workflow version requires a different reviewed schema"
                .into(),
        );
    }
    authorize(pool, grant_id, binding_id).await?;
    validate(&binding.input_schema, input, "input")?;
    let discovered = client::discover(
        &binding.endpoint,
        binding.secret_key.as_deref(),
        &binding.tool_name,
    )
    .await?;
    let current_fingerprint = fingerprint::calculate(
        &discovered.server_identity,
        &binding.tool_name,
        &discovered.input_schema,
        discovered.output_schema.as_ref(),
    )?;
    if current_fingerprint != binding.schema_fingerprint {
        return Err("external-tool-schema-changed: binding requires review".into());
    }
    let arguments = input
        .as_object()
        .cloned()
        .ok_or("external-tool-input-invalid: tool arguments must be an object")?;
    let result = client::call(
        &binding.endpoint,
        binding.secret_key.as_deref(),
        &binding.tool_name,
        arguments,
    )
    .await?;
    validate_result(&binding, &result)?;
    Ok(result)
}

async fn authorize(pool: &SqlitePool, grant_id: &str, binding_id: &str) -> Result<(), String> {
    let mut tx = crate::spaces::db::begin_write(pool).await?;
    let grant = crate::spaces::authorization::load_grant(&mut tx, grant_id).await?;
    tx.rollback().await.map_err(crate::spaces::db::sql_error)?;
    grant.require_egress(
        &format!("external-tool:{binding_id}"),
        crate::clock::unix_time_millis()?,
    )
}

pub(crate) async fn list_enabled(
    pool: &SqlitePool,
    space_id: &str,
) -> Result<Vec<ExternalToolBinding>, String> {
    Ok(repository::list(pool, space_id)
        .await?
        .into_iter()
        .filter(|binding| binding.enabled)
        .collect())
}

fn validate_result(binding: &ExternalToolBinding, result: &Value) -> Result<(), String> {
    let size = serde_json::to_vec(result)
        .map_err(crate::spaces::db::json_error)?
        .len();
    if size > MAX_RESULT_BYTES {
        return Err("external-tool-result-too-large: result exceeds 1 MiB".into());
    }
    if let Some(schema) = binding.output_schema.as_ref() {
        let structured = result
            .get("structuredContent")
            .ok_or("external-tool-output-invalid: structured content is required")?;
        validate(schema, structured, "output")?;
    }
    Ok(())
}

fn validate(schema: &Value, value: &Value, boundary: &str) -> Result<(), String> {
    let validator = jsonschema::validator_for(schema)
        .map_err(|error| format!("external-tool-schema-invalid: {error}"))?;
    validator
        .validate(value)
        .map_err(|error| format!("external-tool-{boundary}-invalid: {error}"))
}

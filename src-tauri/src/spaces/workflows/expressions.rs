const MAX_EXPRESSION_BYTES: usize = 4_096;
const MAX_EXPRESSION_TOKENS: usize = 512;
const MAX_EXPRESSION_NESTING: usize = 32;
const MAX_RESULT_BYTES: usize = 1_048_576;
const MAX_RESULT_ITEMS: usize = 10_000;
const MAX_EVALUATION_MILLIS: u128 = 50;

pub(super) fn validate(source: &str) -> Result<(), String> {
    if source.trim().is_empty() {
        return Err("Workflow expression must not be empty".into());
    }
    if source.len() > MAX_EXPRESSION_BYTES {
        return Err("Workflow expression exceeds 4096 UTF-8 bytes".into());
    }
    validate_complexity(source)?;
    cel::Program::compile(source)
        .map(|_| ())
        .map_err(|error| format!("Workflow CEL expression is invalid: {error}"))
}

pub(crate) fn evaluate(
    source: &str,
    input: &serde_json::Value,
) -> Result<serde_json::Value, String> {
    validate(source)?;
    let program = cel::Program::compile(source)
        .map_err(|error| format!("Workflow CEL expression is invalid: {error}"))?;
    let mut context = cel::Context::default();
    context
        .add_variable("input", input.clone())
        .map_err(|error| format!("Workflow input cannot be represented in CEL: {error}"))?;
    let started = std::time::Instant::now();
    let result = program
        .execute(&context)
        .map_err(|error| format!("Workflow CEL evaluation failed: {error}"))?
        .json()
        .map_err(|error| format!("Workflow CEL result is not JSON-compatible: {error}"))?;
    if started.elapsed().as_millis() > MAX_EVALUATION_MILLIS {
        return Err("Workflow CEL evaluation exceeded 50 milliseconds".into());
    }
    let encoded = serde_json::to_vec(&result)
        .map_err(|error| format!("Workflow CEL result cannot be encoded: {error}"))?;
    if encoded.len() > MAX_RESULT_BYTES || item_count(&result) > MAX_RESULT_ITEMS {
        return Err("Workflow CEL result exceeds its collection or output limit".into());
    }
    Ok(result)
}

fn validate_complexity(source: &str) -> Result<(), String> {
    let mut nesting = 0_usize;
    let mut tokens = 0_usize;
    let mut in_token = false;
    for character in source.chars() {
        if matches!(character, '(' | '[' | '{') {
            nesting += 1;
            if nesting > MAX_EXPRESSION_NESTING {
                return Err("Workflow CEL expression nesting exceeds 32".into());
            }
        } else if matches!(character, ')' | ']' | '}') {
            nesting = nesting.saturating_sub(1);
        }
        let token_character = character.is_alphanumeric() || character == '_';
        if token_character && !in_token {
            tokens += 1;
            if tokens > MAX_EXPRESSION_TOKENS {
                return Err("Workflow CEL expression exceeds 512 tokens".into());
            }
        }
        in_token = token_character;
    }
    Ok(())
}

fn item_count(value: &serde_json::Value) -> usize {
    match value {
        serde_json::Value::Array(values) => {
            values.len() + values.iter().map(item_count).sum::<usize>()
        }
        serde_json::Value::Object(values) => {
            values.len() + values.values().map(item_count).sum::<usize>()
        }
        _ => 0,
    }
}

#[cfg(test)]
mod tests;

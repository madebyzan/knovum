pub(crate) const MAX_SEMANTIC_LIMIT: i64 = 200;

pub(crate) fn validate_query_vector(vector: &[f32], limit: i64) -> Result<(), String> {
    if vector.is_empty() || vector.iter().any(|value| !value.is_finite()) {
        return Err("[SEMANTIC_QUERY_INVALID] query vector must contain finite values".into());
    }
    if vector.iter().map(|value| value * value).sum::<f32>() <= 0.0 {
        return Err("[SEMANTIC_QUERY_INVALID] query vector has zero norm".into());
    }
    if !(1..=MAX_SEMANTIC_LIMIT).contains(&limit) {
        return Err(format!(
            "Semantic result limit must be between 1 and {MAX_SEMANTIC_LIMIT}"
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests;

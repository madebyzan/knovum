use super::*;

fn build_input(vector: Vec<f64>, dims: i64) -> KnowledgeEmbeddingInput {
    KnowledgeEmbeddingInput {
        chunk_id: "c1".into(),
        source_revision: "r1".into(),
        model: "m".into(),
        dimensions: dims,
        vector,
    }
}

#[test]
fn validate_embedding_requires_metadata() {
    let mut bad = build_input(vec![0.1, 0.2], 2);
    bad.chunk_id = String::new();
    assert!(validate_embedding(&bad).is_err());
    let mut bad = build_input(vec![0.1, 0.2], 2);
    bad.source_revision = String::new();
    assert!(validate_embedding(&bad).is_err());
    let mut bad = build_input(vec![0.1, 0.2], 2);
    bad.model = String::new();
    assert!(validate_embedding(&bad).is_err());
}

#[test]
fn validate_embedding_rejects_dimension_mismatch() {
    let bad = build_input(vec![0.1, 0.2, 0.3], 2);
    assert!(validate_embedding(&bad).is_err());
    let bad = build_input(vec![], 0);
    assert!(validate_embedding(&bad).is_err());
}

#[test]
fn validate_embedding_rejects_non_finite_values() {
    let bad = build_input(vec![0.1, f64::NAN], 2);
    assert!(validate_embedding(&bad).is_err());
    let bad = build_input(vec![f64::INFINITY, 0.2], 2);
    assert!(validate_embedding(&bad).is_err());
}

#[test]
fn validate_embedding_accepts_valid_input() {
    assert!(validate_embedding(&build_input(vec![0.1, 0.2], 2)).is_ok());
}

use super::hybrid::reciprocal_rank_fusion;

fn matched(id: &str, score: f64) -> serde_json::Value {
    serde_json::json!({ "chunkId": id, "score": score })
}

#[test]
fn reciprocal_rank_fusion_deduplicates_chunks() {
    let values = reciprocal_rank_fusion(
        vec![matched("a", 7.0), matched("b", 4.0)],
        vec![matched("b", 8.0), matched("a", 3.0), matched("c", 1.0)],
    );
    let ids = values
        .iter()
        .filter_map(|value| value["chunkId"].as_str())
        .collect::<Vec<_>>();
    assert_eq!(ids.len(), 3);
    assert!(ids.contains(&"a") && ids.contains(&"b") && ids.contains(&"c"));
}

#[test]
fn reciprocal_rank_fusion_keeps_both_score_payloads() {
    let values = reciprocal_rank_fusion(vec![matched("a", 7.0)], vec![matched("a", 3.0)]);
    assert_eq!(values.len(), 1);
    assert_eq!(values[0]["lexicalScore"], 7.0);
    assert_eq!(values[0]["semanticScore"], 3.0);
}

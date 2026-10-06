mod usage;

use super::optional_limit;

#[test]
fn optional_budget_limit_distinguishes_absent_and_malformed_values() {
    assert_eq!(
        optional_limit(&serde_json::json!({}), "maxOutputBytes").unwrap(),
        None
    );
    assert_eq!(
        optional_limit(&serde_json::json!({"maxOutputBytes": 0}), "maxOutputBytes").unwrap(),
        Some(0)
    );
    for value in [
        serde_json::json!(null),
        serde_json::json!("100"),
        serde_json::json!(-1),
    ] {
        assert!(optional_limit(
            &serde_json::json!({"maxOutputBytes": value}),
            "maxOutputBytes"
        )
        .is_err());
    }
}

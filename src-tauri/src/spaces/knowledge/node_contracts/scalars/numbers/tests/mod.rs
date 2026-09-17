use serde_json::json;

use super::is_javascript_safe_number;

#[test]
fn accepts_fractional_and_safe_integer_values() {
    assert!(is_javascript_safe_number(&json!(12.5)));
    assert!(is_javascript_safe_number(&json!(9_007_199_254_740_991_i64)));
}

#[test]
fn rejects_integers_that_the_webview_cannot_represent_exactly() {
    assert!(!is_javascript_safe_number(&json!(
        9_007_199_254_740_992_u64
    )));
}

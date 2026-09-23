use serde_json::json;

use super::super::control;
use crate::spaces::{runs::StepOutcome, workflows::definition::ItemErrorPolicy};

#[test]
fn foreach_preserves_order_and_disambiguates_duplicate_keys() {
    let outcome = control::for_each(
        &json!([
            {"itemKey": "source", "value": 1},
            {"itemKey": "source", "value": 2},
            {"id": "third", "value": 3}
        ]),
        3,
        2,
        ItemErrorPolicy::CollectErrors,
    )
    .expect("bounded collection should expand");
    let StepOutcome::Completed { output, .. } = outcome else {
        panic!("foreach must complete")
    };
    let keys = output["foreachItems"]
        .as_array()
        .expect("items")
        .iter()
        .map(|item| item["itemKey"].as_str().expect("key"))
        .collect::<Vec<_>>();
    assert_eq!(keys, ["source", "source:1", "third"]);
    assert_eq!(output["parallelism"], 2);
    assert_eq!(output["errorPolicy"], "collect_errors");
}

#[test]
fn foreach_rejects_unbounded_input() {
    let error = control::for_each(&json!([1, 2]), 1, 1, ItemErrorPolicy::Stop)
        .expect_err("collection above maxItems must fail");
    assert!(error.starts_with("foreach-limit-exceeded:"));
}

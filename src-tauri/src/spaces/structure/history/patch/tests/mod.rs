use super::Change;
use serde::{Deserialize, Serialize};

// A row need not invent a default value just because one side of an edit is absent.
#[derive(Deserialize, Serialize)]
struct Row {
    title: String,
}

#[test]
fn absent_sides_round_trip_without_a_default_row() {
    for (json, before, after) in [
        (r#"{"id":"node","after":{"title":"created"}}"#, false, true),
        (r#"{"id":"node","before":{"title":"deleted"}}"#, true, false),
    ] {
        let change: Change<Row> = serde_json::from_str(json).unwrap();
        assert_eq!(change.before.is_some(), before);
        assert_eq!(change.after.is_some(), after);
        assert_eq!(
            serde_json::to_value(&change).unwrap(),
            serde_json::from_str::<serde_json::Value>(json).unwrap()
        );
    }
}

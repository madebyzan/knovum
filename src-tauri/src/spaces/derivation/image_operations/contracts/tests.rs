
use super::ImageOperation;

#[test]
fn prompt_removal_rejects_missing_and_oversized_descriptions() {
    for prompt in ["  ".to_owned(), "x".repeat(1001)] {
        assert!(ImageOperation::RemoveObjectPrompt { prompt }
            .validate()
            .is_err());
    }
    assert!(ImageOperation::RemoveObjectPrompt {
        prompt: "person in the background".into(),
    }
    .validate()
    .is_ok());
}

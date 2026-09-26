use super::types::{CandidateBinding, CandidateSource};

#[test]
fn captured_bindings_use_camel_case_fields() {
    for (source, field, id) in [
        (
            CandidateSource::InputSlot {
                input_slot_id: "input".into(),
            },
            "inputSlotId",
            "input",
        ),
        (
            CandidateSource::InputSequence {
                input_slot_id: "sequence".into(),
            },
            "inputSlotId",
            "sequence",
        ),
        (
            CandidateSource::StepOutput {
                step_id: "step".into(),
            },
            "stepId",
            "step",
        ),
    ] {
        let binding = CandidateBinding {
            role: "source".into(),
            position: 0,
            revision_basis: "content".into(),
            source,
        };
        let value = serde_json::to_value(&binding).unwrap();
        assert_eq!(value[field], id);
        assert!(value.get("input_slot_id").is_none());
        assert!(value.get("step_id").is_none());
        let decoded: CandidateBinding = serde_json::from_value(value.clone()).unwrap();
        assert_eq!(serde_json::to_value(decoded).unwrap(), value);
    }
}

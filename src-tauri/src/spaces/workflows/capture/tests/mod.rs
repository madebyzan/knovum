use super::types::{CandidateBinding, CandidateSource, CaptureInputRole};
use crate::spaces::derivation::input_revision::RevisionBasis;

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
            role: CaptureInputRole::Source,
            position: 0,
            revision_basis: RevisionBasis::Knowledge,
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

#[test]
fn captured_bindings_reject_unknown_role_and_revision_basis() {
    let valid = serde_json::json!({
        "role": "source",
        "position": 0,
        "revisionBasis": "knowledge",
        "sourceType": "inputSlot",
        "inputSlotId": "input"
    });
    let mut unknown_role = valid.clone();
    unknown_role["role"] = "unknown".into();
    assert!(serde_json::from_value::<CandidateBinding>(unknown_role).is_err());
    let mut unknown_basis = valid;
    unknown_basis["revisionBasis"] = "content".into();
    assert!(serde_json::from_value::<CandidateBinding>(unknown_basis).is_err());
}

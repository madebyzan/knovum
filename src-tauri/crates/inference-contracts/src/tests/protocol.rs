use crate::{
    image_wire::TensorFileReference,
    runtime::{RuntimeAckChunkParams, RuntimeHealthParams},
    speech_wire::{alignment_execution_id, AsrEngine},
};
use serde_json::json;

#[test]
fn control_frames_reject_unknown_fields_and_negative_epochs() {
    assert!(serde_json::from_value::<RuntimeHealthParams>(json!({
        "sessionId": "session", "obsoleteRuntime": "python"
    }))
    .is_err());
    let mut acknowledgement = json!({
        "requestId": "request", "attemptEpoch": 2, "seq": 1, "consumedBytes": 4
    });
    let decoded: RuntimeAckChunkParams = serde_json::from_value(acknowledgement.clone()).unwrap();
    assert_eq!(decoded.attempt_epoch, 2);
    assert_eq!(decoded.consumed_bytes, 4);
    acknowledgement["attemptEpoch"] = json!(-1);
    assert!(serde_json::from_value::<RuntimeAckChunkParams>(acknowledgement).is_err());
}

#[test]
fn tensor_wire_requires_exact_rank_unsigned_shape_and_current_dtype() {
    let valid = json!({
        "fileName": "input.f32", "byteLength": 1228800,
        "sha256": "a".repeat(64), "shape": [1, 3, 320, 320],
        "dataType": "float32_little_endian"
    });
    let tensor: TensorFileReference = serde_json::from_value(valid.clone()).unwrap();
    assert_eq!(tensor.shape, [1, 3, 320, 320]);
    for shape in [json!([1, 3, 320]), json!([1, 3, -1, 320])] {
        let mut invalid = valid.clone();
        invalid["shape"] = shape;
        assert!(serde_json::from_value::<TensorFileReference>(invalid).is_err());
    }
    let mut invalid = valid;
    invalid["dataType"] = json!("float32");
    assert!(serde_json::from_value::<TensorFileReference>(invalid).is_err());
}

#[test]
fn native_engine_and_alignment_identities_do_not_reinterpret_obsolete_values() {
    for engine in ["faster_whisper", "mlx_whisper"] {
        assert!(serde_json::from_value::<AsrEngine>(json!(engine)).is_err());
    }
    assert!(matches!(
        serde_json::from_value::<AsrEngine>(json!("whisper_cpp")).unwrap(),
        AsrEngine::WhisperCpp
    ));
    for (spec, precision) in [
        ("qwen3-forced-aligner-0.6b-f16-native-v2", "f16"),
        ("qwen3-forced-aligner-0.6b-q8_0-v1", "q8_0"),
    ] {
        for device in ["cpu", "mps", "vulkan"] {
            assert_eq!(
                alignment_execution_id(spec, device).unwrap(),
                format!("qwen3-forced-aligner-0.6b-{precision}-{device}-native-v2")
            );
        }
        assert!(alignment_execution_id(spec, "cuda").is_none());
    }
    assert!(alignment_execution_id("qwen3-forced-aligner-0.6b-v1", "cpu").is_none());
}

use crate::{
    image_wire::TensorFileReference,
    runtime::{RuntimeAckChunkParams, RuntimeHealthParams},
    speech_wire::AsrEngine,
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
fn inference_engine_identities_do_not_reinterpret_obsolete_values() {
    for engine in ["faster_whisper", "mlx_whisper"] {
        assert!(serde_json::from_value::<AsrEngine>(json!(engine)).is_err());
    }
    assert!(matches!(
        serde_json::from_value::<AsrEngine>(json!("whisper_cpp")).unwrap(),
        AsrEngine::WhisperCpp
    ));
}

#[test]
fn layer_discovery_bounds_are_shared_by_both_sides_of_the_worker_boundary() {
    use crate::image_compute::{validate_discovery, MAX_DISCOVERED_LAYERS, MAX_DISCOVERY_SEEDS};
    validate_discovery(1, 1).unwrap();
    validate_discovery(MAX_DISCOVERY_SEEDS, MAX_DISCOVERED_LAYERS).unwrap();
    for (seeds, layers) in [
        (0, MAX_DISCOVERED_LAYERS),
        (MAX_DISCOVERY_SEEDS + 1, MAX_DISCOVERED_LAYERS),
        (1, 0),
        (1, MAX_DISCOVERED_LAYERS + 1),
    ] {
        assert!(
            validate_discovery(seeds, layers).is_err(),
            "{seeds} seeds and {layers} layers are out of bounds"
        );
    }
    // Ranks share a u8 label map with the unassigned 0, and a caller must be able to explore the
    // canvas beyond the layers it can publish.
    const { assert!(MAX_DISCOVERED_LAYERS < u8::MAX) };
    const { assert!(MAX_DISCOVERY_SEEDS > MAX_DISCOVERED_LAYERS as usize) };
}

#[test]
fn image_specs_match_provenance_and_processor_config() {
    use crate::image_compute::ImageModelVariant;
    use crate::image_spec::{
        Geometry, OutputField, OutputHead, OutputNormalization, Scale, Standardization,
        BIREFNET_SPEC, DEPTH_ANYTHING_V2_SPEC, HIGH_RES_SEGMENTATION_RESOLUTION, ISNET_ANIME_SPEC,
        ISNET_DIS_SPEC, RMBG14_SPEC,
    };
    use serde_json::Value;

    // L10: Verify Depth Anything V2 spec matches the shipped preprocessor_config.json metadata
    let processor_raw = r#"{
        "do_normalize": true,
        "do_pad": false,
        "do_rescale": true,
        "do_resize": true,
        "ensure_multiple_of": 14,
        "image_mean": [0.485, 0.456, 0.406],
        "image_processor_type": "DPTImageProcessor",
        "image_std": [0.229, 0.224, 0.225],
        "keep_aspect_ratio": true,
        "resample": 3,
        "rescale_factor": 0.00392156862745098,
        "size": {"height": 518, "width": 518},
        "size_divisor": null
    }"#;
    let config: Value = serde_json::from_str(processor_raw).unwrap();
    let expected_patch = config["ensure_multiple_of"].as_u64().unwrap() as u32;
    let expected_edge = config["size"]["height"].as_u64().unwrap() as u32;
    assert_eq!(
        DEPTH_ANYTHING_V2_SPEC.geometry,
        Geometry::ShortestEdgePatchAligned {
            edge: expected_edge,
            patch: expected_patch,
        }
    );
    let mean: Vec<f32> = config["image_mean"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_f64().unwrap() as f32)
        .collect();
    let std: Vec<f32> = config["image_std"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_f64().unwrap() as f32)
        .collect();
    assert_eq!(
        DEPTH_ANYTHING_V2_SPEC.standardization,
        Standardization::ImageNet {
            mean: [mean[0], mean[1], mean[2]],
            std: [std[0], std[1], std[2]],
        }
    );
    assert_eq!(DEPTH_ANYTHING_V2_SPEC.scale, Scale::UnitScale);

    // High-resolution segmentation specs invariant checks
    for (spec, expected_head) in [
        (&BIREFNET_SPEC, OutputHead::Single),
        (&RMBG20_SPEC, OutputHead::Single),
        (&BIREFNET_PORTRAIT_SPEC, OutputHead::Single),
        (&RMBG14_SPEC, OutputHead::Single),
        (&ISNET_DIS_SPEC, OutputHead::FirstOfSupervision),
        (&ISNET_ANIME_SPEC, OutputHead::FirstOfSupervision),
    ] {
        assert_eq!(
            spec.geometry,
            Geometry::Stretch {
                width: HIGH_RES_SEGMENTATION_RESOLUTION,
                height: HIGH_RES_SEGMENTATION_RESOLUTION,
            }
        );
        assert_eq!(spec.output_head, expected_head);
        assert_eq!(
            spec.output_field,
            OutputField::RawScalarField {
                normalize: OutputNormalization::MinMax,
            }
        );
    }

    // Ensure all defined variant specs resolve and have non-empty provenance
    for variant in [
        ImageModelVariant::RealEsrganGeneralX4v3,
        ImageModelVariant::RealEsrganX4plus,
        ImageModelVariant::Migan,
        ImageModelVariant::Lama,
        ImageModelVariant::EfficientSamTi,
        ImageModelVariant::EfficientSamS,
        ImageModelVariant::DepthAnythingV2SmallUint8,
        ImageModelVariant::DepthAnythingV2Small,
    ] {
        let spec = variant.spec().expect("variant must have a frozen spec");
        assert!(!spec.provenance.reference.is_empty());
        assert!(!spec.provenance.revision.is_empty());
    }
}

use crate::spaces::speech::voice_candidates::VoiceModel;
use serde_json::json;

mod fixture;

#[test]
fn accepts_redacted_designed_and_selected_reference_candidate_packages() {
    for model in [VoiceModel::VoxCpm2Q8, VoiceModel::VoxCpm2] {
        for selected in [false, true] {
            let (manifest, assets) = fixture::package(selected, model);
            let request = crate::spaces::package::remap::prepare(
                &manifest.to_string(),
                &assets,
                "imported-space",
            )
            .unwrap();
            super::super::validate_request_shape(&request).unwrap();
        }
    }
}

#[test]
fn rejects_tampered_candidate_execution_provenance() {
    for mutation in [
        "workflow",
        "attempt",
        "config",
        "authorization",
        "output",
        "seed",
    ] {
        let (mut manifest, assets) = fixture::package(false, VoiceModel::VoxCpm2);
        match mutation {
            "workflow" => manifest["runs"][0]["inputSnapshot"]["workflow"] = json!("other"),
            "attempt" => manifest["runAttempts"][0]["requestDigest"] = json!("0".repeat(64)),
            "config" => {
                manifest["runSteps"][0]["configuration"]["request"]["language"] = json!("en")
            }
            "authorization" => {
                manifest["voiceCandidates"][0]["authorization"]["declared"] = json!(false)
            }
            "seed" => manifest["voiceCandidates"][0]["synthesisConfig"]["seed"] = json!(1),
            "output" => {
                manifest["runAttempts"][0]["result"]["audio"]["sha256"] = json!("f".repeat(64))
            }
            _ => unreachable!(),
        }
        let result = crate::spaces::package::remap::prepare(
            &manifest.to_string(),
            &assets,
            "imported-space",
        )
        .and_then(|request| super::super::validate_request_shape(&request));
        assert!(
            result.is_err(),
            "tampered {mutation} provenance was accepted"
        );
    }
}

#[test]
fn rejects_unknown_selected_revision_and_misplaced_candidate_contract_fields() {
    let (mut manifest, assets) = fixture::package(true, VoiceModel::VoxCpm2);
    manifest["voiceCandidates"][0]["selectedVoiceRevisionId"] = json!("missing");
    assert!(crate::spaces::package::remap::prepare(
        &manifest.to_string(),
        &assets,
        "imported-space"
    )
    .is_err());

    let (mut manifest, assets) = fixture::package(true, VoiceModel::VoxCpm2);
    manifest["voiceCandidates"][0]["inputSourceRevision"] =
        json!(format!("sha256:{}", "9".repeat(64)));
    let result =
        crate::spaces::package::remap::prepare(&manifest.to_string(), &assets, "imported-space")
            .and_then(|request| super::super::validate_request_shape(&request));
    assert!(result.is_err());

    let (mut manifest, assets) = fixture::package(true, VoiceModel::VoxCpm2);
    manifest["voiceRevisions"][0]["inputSourceRevision"] = json!(null);
    assert!(crate::spaces::package::remap::prepare(
        &manifest.to_string(),
        &assets,
        "imported-space"
    )
    .is_err());
}

#[test]
fn rejects_reference_selection_outside_supported_interval() {
    let (mut manifest, assets) = fixture::package(true, VoiceModel::VoxCpm2);
    manifest["runSteps"][0]["configuration"]["request"]["source"]["endMs"] = json!(1000);
    let result =
        crate::spaces::package::remap::prepare(&manifest.to_string(), &assets, "imported-space")
            .and_then(|request| super::super::validate_request_shape(&request));
    assert!(result.is_err());
}

#[test]
fn preserves_selected_moss_reference_without_inventing_voxcpm_transcript_controls() {
    let (manifest, assets) = fixture::package(true, VoiceModel::MossNano);
    let request =
        crate::spaces::package::remap::prepare(&manifest.to_string(), &assets, "imported-space")
            .unwrap();
    super::super::validate_request_shape(&request).unwrap();
}

#[test]
fn rejects_model_and_selected_clip_provenance_changes() {
    for mutation in [
        "request_model",
        "output_model",
        "engine",
        "clip_revision",
        "clip_duration",
        "input_length",
    ] {
        let (mut manifest, assets) = fixture::package(true, VoiceModel::VoxCpm2);
        match mutation {
            "request_model" => {
                manifest["runSteps"][0]["configuration"]["request"]["model"] = json!("mossNano")
            }
            "output_model" => manifest["runSteps"][0]["output"]["model"] = json!("mossNano"),
            "engine" => manifest["voiceRevisions"][0]["engineId"] = json!("moss_nano"),
            "clip_revision" | "clip_duration" => {
                let key = if mutation == "clip_revision" {
                    "sourceRevision"
                } else {
                    "durationMs"
                };
                let value = if mutation == "clip_revision" {
                    json!(format!("sha256:{}", "1".repeat(64)))
                } else {
                    json!(9000)
                };
                manifest["runs"][0]["inputSnapshot"]["referenceSource"][key] = value.clone();
                manifest["runSteps"][0]["configuration"]["referenceSource"][key] = value;
            }
            "input_length" => manifest["voiceCandidates"][0]["inputByteLength"] = json!(32000),
            _ => unreachable!(),
        }
        let result = crate::spaces::package::remap::prepare(
            &manifest.to_string(),
            &assets,
            "imported-space",
        )
        .and_then(|request| super::super::validate_request_shape(&request));
        assert!(
            result.is_err(),
            "tampered {mutation} provenance was accepted"
        );
    }
}

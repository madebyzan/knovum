use crate::local_resources::model_catalog::LocalModelId;
use serde_json::json;

#[test]
fn image_actions_bind_the_operation_and_model_specification() {
    let model = LocalModelId::RealEsrganGeneralX4v3;
    let parameters = json!({"title":"Upscaled image", "operation":{"kind":"upscale", "targetLongEdge":2048},
        "execution":{"type":"local", "model":model, "specId":model.image_spec().unwrap()}});
    let upscale = super::super::require("upscale-image").unwrap();
    assert!(super::super::validate_parameters(upscale, &parameters).is_ok());
    let depth = super::super::require("generate-image-depth").unwrap();
    assert!(super::super::validate_parameters(depth, &parameters).is_err());
    for field in ["model", "specId"] {
        let mut invalid = parameters.clone();
        invalid["execution"][field] = if field == "model" {
            json!(LocalModelId::Migan)
        } else {
            json!("obsolete-spec")
        };
        assert!(super::super::validate_parameters(upscale, &invalid).is_err());
    }
    let mut invalid_size = parameters;
    invalid_size["operation"]["targetLongEdge"] = json!(3000);
    assert!(super::super::validate_parameters(upscale, &invalid_size).is_err());
}

#[test]
fn local_inpainting_does_not_claim_text_guided_generation() {
    let model = LocalModelId::Migan;
    let definition = super::super::require("remove-image-object").unwrap();
    let parameters = json!({"title":"Repaired image", "operation":{"kind":"remove-object-prompt", "prompt":"Remove the chair"},
        "execution":{"type":"local", "model":model, "specId":model.image_spec().unwrap()}});
    assert!(super::super::validate_parameters(definition, &parameters).is_err());
}

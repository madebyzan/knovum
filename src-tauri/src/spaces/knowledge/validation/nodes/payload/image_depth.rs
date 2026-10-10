use serde_json::Value;

pub(super) fn validate(value: Option<&Value>) -> Result<(), String> {
    let Some(value) = value else {
        return Ok(());
    };
    let object = value.as_object().ok_or("Image depth must be an object")?;
    super::super::validation::reject_unknown(object, ["encoding", "modelSpecId"], "Image depth")?;
    if super::super::scalars::required_string(object, "encoding", 64)?
        != "relative-inverse-depth-u16-v1"
    {
        return Err("Image depth encoding is invalid".into());
    }
    let spec = super::super::scalars::required_string(object, "modelSpecId", 256)?;
    if !crate::local_resources::model_catalog::MODELS
        .iter()
        .any(|model| {
            model.capability()
                == crate::local_resources::model_catalog::LocalModelCapability::ImageDepth
                && model.image_spec().is_ok_and(|expected| expected == spec)
        })
    {
        return Err("Image depth specification is unknown".into());
    }
    Ok(())
}

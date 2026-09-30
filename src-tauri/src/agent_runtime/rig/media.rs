use rig_agent::core::message::{
    AdditionalParams, DocumentSourceKind, Image, ImageMediaType, MimeType,
};

use crate::ai::contracts::{ModelBinarySource, ModelContentPart};

pub(super) fn to_image(part: ModelContentPart) -> Result<Image, String> {
    let ModelContentPart::Binary {
        media_type,
        source,
        name,
    } = part
    else {
        return Err("assistant-media-invalid: expected an image".into());
    };
    let media_type = ImageMediaType::from_mime_type(&media_type)
        .ok_or("assistant-media-unsupported: expected a supported image media type")?;
    Ok(Image {
        data: match source {
            ModelBinarySource::Base64 { data } => DocumentSourceKind::Base64(data),
            ModelBinarySource::Url { url } => DocumentSourceKind::Url(url),
        },
        media_type: Some(media_type),
        detail: None,
        additional_params: AdditionalParams::from_entries(
            name.map(|name| ("name", serde_json::json!(name))),
        ),
    })
}

pub(super) fn from_image(image: Image) -> Result<ModelContentPart, String> {
    let media_type = image
        .media_type
        .ok_or("assistant-media-invalid: image MIME type is missing")?;
    let source = match image.data {
        DocumentSourceKind::Base64(data) => ModelBinarySource::Base64 { data },
        DocumentSourceKind::Url(url) => ModelBinarySource::Url { url },
        _ => return Err("assistant-media-unsupported: image source cannot be preserved".into()),
    };
    Ok(ModelContentPart::Binary {
        media_type: media_type.to_mime_type().to_owned(),
        source,
        name: image.additional_params.and_then(|params| {
            params
                .get("name")
                .and_then(|name| name.as_str())
                .map(str::to_owned)
        }),
    })
}

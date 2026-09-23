use serde_json::{json, Value};
use sha2::{Digest, Sha256};

#[derive(Clone, Copy)]
pub enum RevisionBasis {
    Asset,
    Knowledge,
    Rendered,
}

impl RevisionBasis {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Asset => "asset",
            Self::Knowledge => "knowledge",
            Self::Rendered => "rendered",
        }
    }

    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "asset" => Ok(Self::Asset),
            "knowledge" => Ok(Self::Knowledge),
            "rendered" => Ok(Self::Rendered),
            _ => Err("Derivation input revision basis is invalid".into()),
        }
    }
}

pub fn resolve(kind: &str, data: &Value, basis: RevisionBasis) -> Result<(String, Value), String> {
    let field = match basis {
        RevisionBasis::Asset | RevisionBasis::Rendered => "sourceRevision",
        RevisionBasis::Knowledge => "knowledgeRevision",
    };
    let base = data
        .get(field)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or("Derivation input revision is unavailable")?;
    let snapshot = match basis {
        RevisionBasis::Asset | RevisionBasis::Knowledge => Value::Null,
        RevisionBasis::Rendered => match kind {
            "image" => json!({ "imageEdit": data.get("imageEdit").unwrap_or(&Value::Null) }),
            "video" => {
                json!({ "videoEditProject": data.get("videoEditProject").ok_or("Video edit project is unavailable")? })
            }
            _ => return Err("Rendered revision requires an image or video".into()),
        },
    };
    Ok((from_snapshot(kind, base, basis, &snapshot)?, snapshot))
}

pub fn from_snapshot(
    kind: &str,
    base: &str,
    basis: RevisionBasis,
    snapshot: &Value,
) -> Result<String, String> {
    match basis {
        RevisionBasis::Asset | RevisionBasis::Knowledge if snapshot.is_null() => Ok(base.into()),
        RevisionBasis::Rendered
            if snapshot.as_object().is_some_and(|fields| {
                fields.len() == 1
                    && ((kind == "image" && fields.contains_key("imageEdit"))
                        || (kind == "video" && fields.contains_key("videoEditProject")))
            }) =>
        {
            if kind == "image" {
                crate::images::validate_edit(&snapshot["imageEdit"])?;
            } else {
                let project: super::video_edit::VideoEditProject =
                    serde_json::from_value(snapshot["videoEditProject"].clone())
                        .map_err(|error| format!("Invalid video edit project: {error}"))?;
                if project.version != 1 || project.source_revision != base {
                    return Err("Video edit project does not match its asset".into());
                }
            }
            let payload =
                serde_json::to_vec(&json!({ "kind": kind, "asset": base, "edit": snapshot }))
                    .map_err(|error| error.to_string())?;
            let digest = Sha256::digest(payload);
            Ok(format!("rendered1:{digest:x}"))
        }
        _ => Err("Derivation input snapshot does not match its revision basis".into()),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{resolve, RevisionBasis};

    #[test]
    fn rendered_revision_tracks_image_edits_but_not_annotations() {
        let mut image =
            json!({"sourceRevision": "sha256:asset", "title": "First", "imageEdit": null});
        let original = resolve("image", &image, RevisionBasis::Rendered).unwrap();
        image["title"] = json!("Renamed");
        image["imageComments"] = json!([{"body": "Comment"}]);
        assert_eq!(
            resolve("image", &image, RevisionBasis::Rendered).unwrap(),
            original
        );

        image["imageEdit"] = json!({
            "version": 5,
            "crop": {"x": 0.0, "y": 0.0, "width": 1.0, "height": 1.0},
            "rotation": 90.0,
            "flipHorizontal": false,
            "flipVertical": false,
            "adjustments": {
                "exposure": 0.0, "contrast": 0.0, "highlights": 0.0,
                "shadows": 0.0, "whites": 0.0, "blacks": 0.0,
                "saturation": 0.0, "warmth": 0.0, "sharpness": 0.0,
                "vignette": 0.0, "vibrance": 0.0, "tint": 0.0,
                "clarity": 0.0, "grain": 0.0, "softLight": 0.0, "glow": 0.0
            },
            "eraserStrokes": []
        });
        assert_ne!(
            resolve("image", &image, RevisionBasis::Rendered).unwrap().0,
            original.0
        );
        assert_eq!(
            resolve("image", &image, RevisionBasis::Asset).unwrap().0,
            "sha256:asset"
        );
    }
}

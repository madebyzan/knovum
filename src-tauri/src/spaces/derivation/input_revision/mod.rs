use serde_json::{json, Value};
use sha2::{Digest, Sha256};

#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    Ord,
    PartialEq,
    PartialOrd,
    serde::Deserialize,
    serde::Serialize,
    ts_rs::TS,
)]
#[serde(rename_all = "lowercase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
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

pub fn resolve(
    kind: &str,
    payload: &Value,
    basis: RevisionBasis,
) -> Result<(String, Value), String> {
    let field = match basis {
        RevisionBasis::Asset | RevisionBasis::Rendered => "sourceRevision",
        RevisionBasis::Knowledge => "knowledgeRevision",
    };
    let base = payload
        .get(field)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or("Derivation input revision is unavailable")?;
    let snapshot = match basis {
        RevisionBasis::Asset | RevisionBasis::Knowledge => Value::Null,
        RevisionBasis::Rendered => match kind {
            "image" => json!({ "imageEdit": payload.get("imageEdit").unwrap_or(&Value::Null) }),
            "video" => {
                json!({ "videoEditProject": payload.get("videoEditProject").ok_or("Video edit project is unavailable")? })
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
                if project.version != super::video_edit::PROJECT_VERSION
                    || project.source_revision != base
                {
                    return Err("Video edit project does not match its asset".into());
                }
            }
            let payload =
                serde_json::to_vec(&json!({ "kind": kind, "asset": base, "edit": snapshot }))
                    .map_err(|error| error.to_string())?;
            let digest = Sha256::digest(payload);
            Ok(format!("rendered1:{digest}", digest = hex::encode(digest)))
        }
        _ => Err("Derivation input snapshot does not match its revision basis".into()),
    }
}

#[cfg(test)]
mod tests;

use std::collections::{BTreeMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::ai::tasks::TaskTargetRef;

#[derive(Deserialize, Serialize, ts_rs::TS)]
#[serde(untagged)]
pub enum GenerationParameterValue {
    String(String),
    Number(f64),
    Boolean(bool),
}

#[derive(Deserialize, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GenerationPreset {
    pub id: String,
    pub name: String,
    pub action_id: String,
    pub target: TaskTargetRef,
    pub parameters: BTreeMap<String, GenerationParameterValue>,
}

#[derive(Deserialize, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreationLibrary {
    pub generation_presets: Vec<GenerationPreset>,
}

impl CreationLibrary {
    pub fn validate(&self) -> Result<(), String> {
        let mut identities = HashSet::new();
        for preset in &self.generation_presets {
            if preset.id.is_empty() || preset.name.is_empty() || preset.action_id.is_empty() {
                return Err("Generation preset identity cannot be empty".into());
            }
            if !identities.insert(preset.id.as_str()) {
                return Err("Generation preset identities must be unique".into());
            }
            match &preset.target {
                TaskTargetRef::Model {
                    connection_id,
                    model_id,
                } if connection_id.is_empty() || model_id.is_empty() => {
                    return Err("Generation preset model target is invalid".into());
                }
                _ => {}
            }
        }
        Ok(())
    }
}

use serde::{Deserialize, Serialize};

pub const SCHEMA_VERSION: u8 = 1;
pub const SIDEBAR_WIDTH_MIN: u16 = 200;
pub const SIDEBAR_WIDTH_MAX: u16 = 320;
pub const RIGHT_SIDEBAR_WIDTH_MIN: u16 = 300;
pub const RIGHT_SIDEBAR_WIDTH_MAX: u16 = 480;

#[derive(Deserialize, Serialize, schemars::JsonSchema, ts_rs::TS)]
#[serde(rename_all = "lowercase")]
pub enum ThemePreference {
    System,
    Light,
    Dark,
}

#[derive(Deserialize, Serialize, schemars::JsonSchema, ts_rs::TS)]
#[serde(rename_all = "lowercase")]
pub enum DensityPreference {
    Compact,
    Comfortable,
}

#[derive(Deserialize, Serialize, schemars::JsonSchema, ts_rs::TS)]
#[serde(rename_all = "lowercase")]
pub enum TextSizePreference {
    Small,
    Default,
    Large,
}

#[derive(Clone, Copy, Deserialize, Serialize, schemars::JsonSchema, ts_rs::TS)]
pub enum LocalePreference {
    #[serde(rename = "system")]
    System,
    #[serde(rename = "en")]
    English,
    #[serde(rename = "zh-CN")]
    SimplifiedChinese,
    #[serde(rename = "zh-TW")]
    TraditionalChinese,
    #[serde(rename = "ja")]
    Japanese,
    #[serde(rename = "ko")]
    Korean,
    #[serde(rename = "fr")]
    French,
    #[serde(rename = "de")]
    German,
    #[serde(rename = "es")]
    Spanish,
    #[serde(rename = "pt-BR")]
    BrazilianPortuguese,
    #[serde(rename = "it")]
    Italian,
    #[serde(rename = "ru")]
    Russian,
}

#[derive(Deserialize, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields, default)]
pub struct WorkbenchPreferences {
    pub sidebar_visible: bool,
    pub sidebar_width: u16,
    pub right_sidebar_visible: bool,
    pub right_sidebar_width: u16,
}

impl Default for WorkbenchPreferences {
    fn default() -> Self {
        Self {
            sidebar_visible: true,
            sidebar_width: 224,
            right_sidebar_visible: false,
            right_sidebar_width: 380,
        }
    }
}

#[derive(Deserialize, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Preferences {
    pub theme: ThemePreference,
    pub density: DensityPreference,
    pub text_size: TextSizePreference,
    pub locale: LocalePreference,
    #[serde(default = "Preferences::default_background_task_notifications")]
    pub background_task_notifications: bool,
    #[serde(default)]
    pub workbench: WorkbenchPreferences,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            theme: ThemePreference::System,
            density: DensityPreference::Compact,
            text_size: TextSizePreference::Default,
            locale: LocalePreference::System,
            background_task_notifications: Self::default_background_task_notifications(),
            workbench: WorkbenchPreferences::default(),
        }
    }
}

impl Preferences {
    fn default_background_task_notifications() -> bool {
        true
    }

    pub fn validate(&self) -> Result<(), String> {
        if !(SIDEBAR_WIDTH_MIN..=SIDEBAR_WIDTH_MAX).contains(&self.workbench.sidebar_width)
            || !(RIGHT_SIDEBAR_WIDTH_MIN..=RIGHT_SIDEBAR_WIDTH_MAX)
                .contains(&self.workbench.right_sidebar_width)
        {
            return Err("Workbench preferences contain invalid widths".into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;

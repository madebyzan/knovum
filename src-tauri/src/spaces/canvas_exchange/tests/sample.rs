use std::path::Path;

use super::super::{
    contracts::{JsonCanvasDocumentV1, JsonCanvasNodeV1},
    validation,
};
use super::*;

const LOCALES: [LocalePreference; 11] = [
    LocalePreference::English,
    LocalePreference::SimplifiedChinese,
    LocalePreference::TraditionalChinese,
    LocalePreference::Japanese,
    LocalePreference::Korean,
    LocalePreference::French,
    LocalePreference::German,
    LocalePreference::Spanish,
    LocalePreference::BrazilianPortuguese,
    LocalePreference::Italian,
    LocalePreference::Russian,
];

#[test]
fn every_language_bundles_a_valid_sample_whose_files_exist() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    for locale in LOCALES {
        let path = root.join(relative_path(locale).unwrap());
        let raw = std::fs::read_to_string(&path).unwrap();
        let document: JsonCanvasDocumentV1 = serde_json::from_str(&raw).unwrap();
        validation::validate(&document).unwrap();
        for node in &document.nodes {
            if let JsonCanvasNodeV1::File(file) = node {
                assert!(path.parent().unwrap().join(&file.file).is_file());
            }
        }
    }
}

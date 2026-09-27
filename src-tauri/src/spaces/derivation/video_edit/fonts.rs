use std::{collections::BTreeMap, sync::OnceLock};

use fontdb::{Database, Style};
use serde::Serialize;

#[derive(Clone, Serialize, ts_rs::TS)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
#[serde(rename_all = "camelCase")]
pub struct InstalledFont {
    pub family: String,
    pub weights: Vec<u16>,
}

static CATALOG: OnceLock<Vec<InstalledFont>> = OnceLock::new();

fn catalog() -> &'static [InstalledFont] {
    CATALOG.get_or_init(|| {
        let mut database = Database::new();
        database.load_system_fonts();
        let mut families = BTreeMap::<String, Vec<u16>>::new();
        for face in database.faces().filter(|face| face.style == Style::Normal) {
            let Some((family, _)) = face.families.first() else {
                continue;
            };
            let weight = face.weight.0;
            if family.is_empty()
                || family
                    .chars()
                    .any(|character| matches!(character, ',' | '\n' | '\r'))
                || !(100..=900).contains(&weight)
            {
                continue;
            }
            let weights = families.entry(family.clone()).or_default();
            if !weights.contains(&weight) {
                weights.push(weight);
            }
        }
        families
            .into_iter()
            .map(|(family, mut weights)| {
                weights.sort_unstable();
                InstalledFont { family, weights }
            })
            .collect()
    })
}

pub fn installed() -> Vec<InstalledFont> {
    catalog().to_vec()
}

pub fn is_available(family: &str, weight: u16) -> bool {
    catalog()
        .iter()
        .any(|font| font.family == family && font.weights.contains(&weight))
}

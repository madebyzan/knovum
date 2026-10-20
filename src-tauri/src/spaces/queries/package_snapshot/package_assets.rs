use std::collections::BTreeMap;

use crate::spaces::{assets, package::PackageAsset};

#[derive(Default)]
pub(super) struct PackageAssets {
    token_by_path: BTreeMap<String, String>,
    assets: Vec<PackageAsset>,
}

impl PackageAssets {
    pub(super) fn reference(&mut self, path: &str) -> String {
        let token = self
            .token_by_path
            .entry(path.to_owned())
            .or_insert_with(|| {
                let token = uuid::Uuid::new_v4().to_string();
                self.assets.push(PackageAsset {
                    token: token.clone(),
                    path: path.to_owned(),
                });
                token
            });
        format!("{}{token}", assets::PACKAGE_ASSET_PREFIX)
    }

    pub(super) fn into_assets(self) -> Vec<PackageAsset> {
        self.assets
    }
}

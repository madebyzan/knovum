//! The file a Space just published, settling onto the node that was waiting for it.
//!
//! Every publisher — a rendered video, a processed image, generated speech — ends in the same
//! place: the node stops being a reservation and starts being this file. Saying that once keeps
//! the facts a node states about its asset identical no matter which work produced it.

use std::path::Path;

use serde_json::{Map, Value};

use crate::spaces::previews::GeneratedPreview;

pub(crate) struct PublishedAsset<'a> {
    pub path: &'a str,
    pub revision: String,
    /// The file's byte size, when its publisher measured it.
    pub size: Option<u64>,
    /// The cover the canvas shows, made from this file. Audio has none.
    pub cover: Option<&'a GeneratedPreview>,
}

impl PublishedAsset<'_> {
    /// Settles this file onto `fields`. Knowledge the output keeps of a source is declared by
    /// `derivation::inheritance`; this is only what the file itself establishes.
    pub(crate) fn settle_onto(
        &self,
        fields: &mut Map<String, Value>,
        now: i64,
    ) -> Result<(), String> {
        let file_name = Path::new(self.path)
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or("Published asset has no file name")?;
        fields.insert("assetPath".into(), self.path.into());
        fields.insert("fileName".into(), file_name.into());
        fields.insert("sourceRevision".into(), self.revision.clone().into());
        if let Some(size) = self.size {
            fields.insert("size".into(), size.into());
        }
        fields.insert("modifiedAt".into(), now.into());
        fields.remove("processingState");
        if let Some(cover) = self.cover {
            cover.settle_onto(fields);
        }
        Ok(())
    }
}

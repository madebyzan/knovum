//! What a derived node keeps of its source.
//!
//! A derived asset publishes its own file, so everything that file establishes — where it lives,
//! its revision, size, cover and geometry — is always the output's own. What remains is knowledge
//! *about* the content: the labels a person gave it, how long it runs, what it sounds like, what
//! is said in it, what its picture shows. Whether that knowledge still holds is a property of the
//! transform rather than of the code that publishes it, so every output made from one source names
//! here what it keeps, and nothing reaches it by accident.
//!
//! Kept knowledge is re-pinned to the asset the output publishes, which is also what tells
//! enrichment there is nothing to read again; knowledge that is not kept is absent, which is what
//! asks enrichment to establish the output's own. Only knowledge that is current for the source is
//! kept: a stale reading stays stale, it is not passed off as describing a new file.
//!
//! Some things are never kept. Provenance (`origin`, `creator`, `externalSource`) says where the
//! source file came from, and a derived file came from a Space. Annotations (`comments`,
//! `imageEdit`, `videoEditProject`) belong to the node they were made on, and an edit is already
//! rendered into whatever was derived from it.
//!
//! Outputs made from knowledge rather than from one asset — generated images, videos and documents,
//! visualizations, the layers a picture is split into — are new works and keep nothing.

use serde_json::{Map, Value};

use crate::spaces::assets::enrichment::jobs::revision::visual_revision;

/// Knowledge about a source's content that a derived output may keep.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Facet {
    /// The labels a person gave the work (`tags`).
    Labels,
    /// How long it runs (`durationSeconds`).
    Timeline,
    /// What it sounds like (`mediaAnalysis`: the measured duration and waveform).
    Sound,
    /// What is said in it (`mediaKnowledge`: the transcript and what was learned from it).
    Speech,
    /// What its picture shows (`visualKnowledge`).
    Picture,
}

/// Every output a Space derives from one source asset.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DerivedOutput {
    SubtitledVideo,
    EditedVideo,
    DubbedVideo,
    Narration,
    SeparatedVocals,
    SeparatedBackground,
    UpscaledImage,
    BackgroundRemovedImage,
    ObjectRemovedImage,
    ExtractedObject,
    DepthMap,
    NewView,
}

impl DerivedOutput {
    pub(crate) const fn keeps(self) -> &'static [Facet] {
        use Facet::{Labels, Picture, Sound, Speech, Timeline};
        match self {
            // Text drawn over the same picture, the same sound and the same words.
            Self::SubtitledVideo => &[Labels, Timeline, Sound, Speech, Picture],
            // The same frames with a new voice; the length is the picture's and is measured anew.
            Self::DubbedVideo => &[Labels, Picture],
            // A re-cut timeline: only what a person said about the work still holds.
            Self::EditedVideo => &[Labels],
            // The same words at the same moments, without what surrounded them.
            Self::SeparatedVocals => &[Speech],
            // The same picture at a higher resolution.
            Self::UpscaledImage => &[Labels, Picture],
            // The same photograph with part of it gone.
            Self::BackgroundRemovedImage | Self::ObjectRemovedImage => &[Labels],
            // A part, a measurement or a new picture of a source, and narration of a text: new works.
            Self::SeparatedBackground
            | Self::ExtractedObject
            | Self::DepthMap
            | Self::NewView
            | Self::Narration => &[],
        }
    }

    /// Whether the output is its source's picture, frame for frame, with something done to it —
    /// detail added, or part of it kept or taken away. Laid over its source it shows exactly what
    /// the transform did, which is how a person judges the result. A measurement of the picture
    /// (depth), another picture of the subject (a new view) and anything heard or watched have
    /// nothing to lay over: they differ everywhere, or not in what the eye can compare.
    pub(crate) const fn compares_with_source(self) -> bool {
        matches!(
            self,
            Self::UpscaledImage
                | Self::BackgroundRemovedImage
                | Self::ObjectRemovedImage
                | Self::ExtractedObject
        )
    }

    /// Carries what this output keeps of `source` into `output`. Runs once the output's own file
    /// has settled onto it, since its revision and cover are what kept knowledge is pinned to.
    pub(crate) fn inherit(self, output: &mut Value, source: &Value) -> Result<(), String> {
        let asset = output
            .get("sourceRevision")
            .and_then(Value::as_str)
            .ok_or("A derived output inherits only once its asset is published")?
            .to_owned();
        // An output without a cover has no picture for visual knowledge to describe.
        let pictures = if self.keeps().contains(&Facet::Picture) {
            visual_revision(output)?
                .map(|pin| Ok::<_, String>((visual_revision(source)?, pin)))
                .transpose()?
        } else {
            None
        };
        let source_asset = source.get("sourceRevision").and_then(Value::as_str);
        let object = output
            .as_object_mut()
            .ok_or("Derived output data is invalid")?;
        for facet in self.keeps() {
            match facet {
                Facet::Labels => keep_labels(object, source),
                Facet::Timeline => copy(object, source, "durationSeconds"),
                Facet::Sound => {
                    keep_pinned(object, source, "mediaAnalysis", source_asset, &asset);
                }
                Facet::Speech => keep_speech(object, source, source_asset, &asset),
                Facet::Picture => {
                    if let Some((current, pin)) = &pictures {
                        keep_pinned(object, source, "visualKnowledge", current.as_deref(), pin);
                    }
                }
            }
        }
        Ok(())
    }
}

/// Labels a person put on the output while it was being made are that output's own and stand.
fn keep_labels(object: &mut Map<String, Value>, source: &Value) {
    if !object.contains_key("tags") {
        copy(object, source, "tags");
    }
}

fn copy(object: &mut Map<String, Value>, source: &Value, field: &str) {
    if let Some(value) = source.get(field) {
        object.insert(field.into(), value.clone());
    }
}

/// Keeps knowledge the source holds for its current content, pinned to what the output publishes.
/// Reports whether it was kept.
fn keep_pinned(
    object: &mut Map<String, Value>,
    source: &Value,
    field: &str,
    current: Option<&str>,
    pin: &str,
) -> bool {
    let Some(mut kept) = source
        .get(field)
        .filter(|knowledge| {
            current.is_some() && knowledge.get("sourceRevision").and_then(Value::as_str) == current
        })
        .cloned()
    else {
        return false;
    };
    let Some(fields) = kept.as_object_mut() else {
        return false;
    };
    fields.insert("sourceRevision".into(), pin.into());
    object.insert(field.into(), kept);
    true
}

/// A transcript names the asset it was recognized from as well, so both pins move together.
fn keep_speech(object: &mut Map<String, Value>, source: &Value, current: Option<&str>, pin: &str) {
    if !keep_pinned(object, source, "mediaKnowledge", current, pin) {
        return;
    }
    if let Some(transcript) = object
        .get_mut("mediaKnowledge")
        .and_then(|knowledge| knowledge.get_mut("transcript"))
        .and_then(Value::as_object_mut)
    {
        transcript.insert("sourceRevision".into(), pin.into());
    }
}

#[cfg(test)]
mod tests;

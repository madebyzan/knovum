use serde_json::{json, Value};

use super::DerivedOutput;
use crate::spaces::assets::enrichment::jobs::revision::visual_revision;

fn source_video() -> Value {
    let mut source = json!({
        "kind":"video","title":"Interview","tags":["interview"],
        "assetPath":"/owned/source.mp4","fileName":"source.mp4","sourceRevision":"asset:1",
        "previewPath":"/previews/source.png","durationSeconds":42,
        "origin":{"kind":"web","url":"https://example.com/v","provider":"example","externalId":"v"},
        "creator":"Someone",
        "mediaAnalysis":{"schemaVersion":1,"sourceRevision":"asset:1","durationMs":42_000,"waveform":[0.5]},
        "mediaKnowledge":{"schemaVersion":1,"sourceRevision":"asset:1","transcript":{"sourceRevision":"asset:1"}}
    });
    let picture = visual_revision(&source).unwrap().unwrap();
    source["visualKnowledge"] =
        json!({"sourceRevision":picture,"description":"Two people talking"});
    source
}

fn published_video() -> Value {
    json!({
        "kind":"video","title":"Interview subtitled","assetPath":"/owned/output.mp4",
        "fileName":"output.mp4","sourceRevision":"asset:2","previewPath":"/previews/output.png"
    })
}

#[test]
fn subtitles_keep_every_reading_of_the_content_pinned_to_the_new_asset() {
    let mut output = published_video();
    DerivedOutput::SubtitledVideo
        .inherit(&mut output, &source_video())
        .unwrap();
    assert_eq!(output["tags"], json!(["interview"]));
    assert_eq!(output["durationSeconds"], 42);
    assert_eq!(output["mediaAnalysis"]["sourceRevision"], "asset:2");
    assert_eq!(output["mediaKnowledge"]["sourceRevision"], "asset:2");
    assert_eq!(
        output["mediaKnowledge"]["transcript"]["sourceRevision"],
        "asset:2"
    );
    assert_eq!(
        output["visualKnowledge"]["sourceRevision"],
        visual_revision(&output).unwrap().unwrap().as_str()
    );
    assert_eq!(
        output["visualKnowledge"]["description"],
        "Two people talking"
    );
}

#[test]
fn provenance_and_the_published_file_are_never_inherited() {
    for derived in [
        DerivedOutput::SubtitledVideo,
        DerivedOutput::DubbedVideo,
        DerivedOutput::EditedVideo,
    ] {
        let mut output = published_video();
        derived.inherit(&mut output, &source_video()).unwrap();
        for field in ["origin", "creator"] {
            assert!(output.get(field).is_none(), "{derived:?} kept {field}");
        }
        assert_eq!(output["assetPath"], "/owned/output.mp4");
        assert_eq!(output["previewPath"], "/previews/output.png");
    }
}

#[test]
fn a_new_voice_keeps_the_picture_but_not_the_sound_or_words() {
    let mut output = published_video();
    DerivedOutput::DubbedVideo
        .inherit(&mut output, &source_video())
        .unwrap();
    assert_eq!(output["tags"], json!(["interview"]));
    assert!(output.get("visualKnowledge").is_some());
    for field in ["mediaAnalysis", "mediaKnowledge", "durationSeconds"] {
        assert!(output.get(field).is_none(), "dubbing kept {field}");
    }
}

#[test]
fn a_recut_keeps_only_labels() {
    let mut output = published_video();
    DerivedOutput::EditedVideo
        .inherit(&mut output, &source_video())
        .unwrap();
    let mut expected = published_video();
    expected["tags"] = json!(["interview"]);
    assert_eq!(output, expected);
}

#[test]
fn knowledge_that_is_stale_for_the_source_is_not_passed_on() {
    let mut source = source_video();
    source["sourceRevision"] = json!("asset:replaced");
    source["previewPath"] = json!("/previews/replaced.png");
    let mut output = published_video();
    DerivedOutput::SubtitledVideo
        .inherit(&mut output, &source)
        .unwrap();
    for field in ["mediaAnalysis", "mediaKnowledge", "visualKnowledge"] {
        assert!(output.get(field).is_none(), "stale {field} was kept");
    }
}

#[test]
fn an_output_without_a_cover_keeps_no_visual_knowledge() {
    let mut output = published_video();
    output.as_object_mut().unwrap().remove("previewPath");
    DerivedOutput::DubbedVideo
        .inherit(&mut output, &source_video())
        .unwrap();
    assert!(output.get("visualKnowledge").is_none());
}

#[test]
fn labels_given_to_the_output_itself_stand() {
    let mut output = published_video();
    output["tags"] = json!(["mine"]);
    DerivedOutput::EditedVideo
        .inherit(&mut output, &source_video())
        .unwrap();
    assert_eq!(output["tags"], json!(["mine"]));
}

#[test]
fn separated_vocals_keep_what_is_said_and_nothing_else() {
    let mut output = json!({
        "kind":"audio","title":"Interview · Vocals","assetPath":"/owned/vocals.wav",
        "fileName":"vocals.wav","sourceRevision":"sha256:vocals","durationSeconds":42
    });
    DerivedOutput::SeparatedVocals
        .inherit(&mut output, &source_video())
        .unwrap();
    assert_eq!(output["mediaKnowledge"]["sourceRevision"], "sha256:vocals");
    assert!(output.get("mediaAnalysis").is_none());
    assert!(output.get("tags").is_none());
}

#[test]
fn an_unpublished_output_cannot_inherit() {
    let mut output = json!({"kind":"video","title":"Pending","processingState":"processing"});
    assert!(DerivedOutput::SubtitledVideo
        .inherit(&mut output, &source_video())
        .is_err());
}

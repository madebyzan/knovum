mod render_smoke;
use super::{
    contracts::{VideoTransition, VideoTransitionKind},
    filter_graph,
    validation::validate_project,
    VideoEditProject,
};

fn project() -> VideoEditProject {
    serde_json::from_str(r#"{
 "version":1,"sourceRevision":"revision",
 "assets":[{"nodeId":"source","sourceRevision":"revision","kind":"video","durationMs":8000,"width":1080,"height":1920}],
 "output":{"width":1080,"height":1920,"frameRate":30},
 "videoTrack":{"clips":[
 {"id":"d51a4384-70ab-4b44-90cc-3bfcf0f5341f","assetId":"source","startMs":1000,"endMs":3000,"volume":1,"fadeInMs":0,"fadeOutMs":0,"transition":null,"transform":{"rotation":90,"flipHorizontal":true,"flipVertical":false,"fit":"contain","zoom":1}},
 {"id":"d51a4384-70ab-4b44-90cc-3bfcf0f5342f","assetId":"source","startMs":5000,"endMs":8000,"volume":0.5,"fadeInMs":0,"fadeOutMs":0,"transition":null,"transform":{"rotation":0,"flipHorizontal":false,"flipVertical":false,"fit":"cover","zoom":1}}]},
 "audioTrack":{"clips":[]},"textTrack":{"items":[]}
}"#).unwrap()
}

#[test]
fn normalizes_each_clip_and_concatenates_audio_and_video() {
    let filter = filter_graph::build(&project(), &[true, true], None).unwrap();
    assert!(filter.contains("trim=duration=2.000"));
    assert!(filter.contains("concat=n=2:v=1:a=0"));
    assert!(filter.contains("transpose=clock,hflip"));
    assert!(filter.contains("fps=30,settb=AVTB"));
    assert!(filter.contains("anull[outa]"));
}
#[test]
fn explicit_silence_keeps_silent_clips_in_sync() {
    let filter = filter_graph::build(&project(), &[false, false], None).unwrap();
    assert!(filter.contains("anullsrc=r=48000:cl=stereo,atrim=duration=2.000"));
    assert!(!filter.contains("[0:a]"));
}
#[test]
fn transition_offsets_and_audio_share_the_overlap() {
    let mut project = project();
    project.video_track.clips[0].transition = Some(VideoTransition {
        kind: VideoTransitionKind::Dissolve,
        duration_ms: 500,
    });
    validate_project(&project, "revision").unwrap();
    assert_eq!(super::timing::duration_ms(&project), 4500);
    let filter = filter_graph::build(&project, &[true, false], None).unwrap();
    assert!(filter.contains("xfade=transition=fade:duration=0.500:offset=1.500"));
    assert!(filter.contains("acrossfade=d=0.500:c1=tri:c2=tri"));
}
#[test]
fn rejects_unknown_assets_and_impossible_overlaps() {
    let mut project = project();
    project.video_track.clips[0].asset_id = "foreign".into();
    assert!(validate_project(&project, "revision").is_err());
    project.video_track.clips[0].asset_id = "source".into();
    project.video_track.clips[0].transition = Some(VideoTransition {
        kind: VideoTransitionKind::WipeLeft,
        duration_ms: 2000,
    });
    assert!(validate_project(&project, "revision").is_err());
}

#[test]
fn rejects_obsolete_and_incomplete_project_shapes() {
    let mut encoded = serde_json::to_value(project()).unwrap();
    encoded["videoTrack"]["clips"][0]
        .as_object_mut()
        .unwrap()
        .remove("transition");
    assert!(serde_json::from_value::<VideoEditProject>(encoded).is_err());
    let mut encoded = serde_json::to_value(project()).unwrap();
    encoded.as_object_mut().unwrap().remove("audioTrack");
    assert!(serde_json::from_value::<VideoEditProject>(encoded).is_err());
}

#[test]
fn saved_snapshot_compares_typed_values_without_losing_edit_conflict_detection() {
    let requested = project();
    let mut data = serde_json::json!({"videoEditProject": requested});
    data["videoEditProject"]["videoTrack"]["clips"][0]["volume"] = serde_json::json!(1);
    data["videoEditProject"]["videoTrack"]["clips"][0]["transform"]["zoom"] = serde_json::json!(1);
    assert!(super::start::validate_saved_project(&data, &requested).is_ok());
    data["videoEditProject"]["videoTrack"]["clips"][0]["volume"] = serde_json::json!(0.75);
    assert!(super::start::validate_saved_project(&data, &requested).is_err());
    data["videoEditProject"]["videoTrack"]["clips"][0]["transition"] =
        serde_json::json!({"kind":"obsolete", "durationMs":500});
    assert!(super::start::validate_saved_project(&data, &requested).is_err());
}

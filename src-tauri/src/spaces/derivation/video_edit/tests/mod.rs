mod render_smoke;
use super::{
    contracts::{VideoTransition, VideoTransitionKind},
    filter_graph, timing,
    validation::validate_project,
    VideoEditProject,
};

pub(super) fn project() -> VideoEditProject {
    serde_json::from_str(
        r##"{
 "version":2,"sourceRevision":"revision",
 "assets":[{"nodeId":"source","sourceRevision":"revision","kind":"video","durationMs":8000,"width":1080,"height":1920}],
 "output":{"width":1080,"height":1920,"frameRate":30,"background":"#000000"},
 "videoTrack":{"clips":[
 {"id":"d51a4384-70ab-4b44-90cc-3bfcf0f5341f","assetId":"source","sourceInMs":1000,"sourceOutMs":3000,"timelineStartMs":0,"volume":1,"fadeInMs":0,"fadeOutMs":0,"transform":{"x":0,"y":0,"scale":1,"rotation":90,"opacity":1,"fit":"contain","flipHorizontal":true,"flipVertical":false}},
 {"id":"d51a4384-70ab-4b44-90cc-3bfcf0f5342f","assetId":"source","sourceInMs":5000,"sourceOutMs":8000,"timelineStartMs":2000,"volume":0.5,"fadeInMs":0,"fadeOutMs":0,"transform":{"x":0,"y":0,"scale":1,"rotation":0,"opacity":1,"fit":"cover","flipHorizontal":false,"flipVertical":false}}]},
 "transitions":[],
 "audioTrack":{"clips":[]},"textTrack":{"items":[]}
}"##,
    )
    .unwrap()
}

fn dissolve(from_clip_id: &str, duration_ms: u64) -> VideoTransition {
    VideoTransition {
        id: "9f3a1f3c-54f0-4c2f-9a1e-6f5c9a2b7d01".into(),
        from_clip_id: from_clip_id.to_owned(),
        kind: VideoTransitionKind::Dissolve,
        duration_ms,
    }
}

#[test]
fn composes_each_clip_over_the_project_background_and_cuts_hard() {
    let project = project();
    validate_project(&project, "revision").unwrap();
    assert_eq!(timing::duration_ms(&project), 5000);
    let filter = filter_graph::build(&project, &[true, true], None).unwrap();
    assert!(filter.contains("split=2[bg0][bg1]"));
    assert!(filter.contains("trim=duration=2.000"));
    assert!(filter.contains("rotate=a=1.570796"));
    assert!(filter.contains("hflip"));
    assert!(filter.contains("overlay=x=(main_w/2+0)-overlay_w/2"));
    assert!(filter.contains("concat=n=2:v=1:a=0,fps=30,settb=AVTB"));
    assert!(filter.contains("adelay=2000:all=1"));
    assert!(filter.contains("amix=inputs=2:duration=longest:normalize=0"));
}

#[test]
fn silent_sources_leave_the_mix_empty_instead_of_padding_filters() {
    let filter = filter_graph::build(&project(), &[false, false], None).unwrap();
    assert!(filter.contains("anullsrc=r=48000:cl=stereo,atrim=duration=5.000"));
    assert!(!filter.contains("[0:a]"));
}

#[test]
fn a_timeline_hole_renders_the_background_between_clips() {
    let mut project = project();
    project.video_track.clips[1].timeline_start_ms = 3000;
    validate_project(&project, "revision").unwrap();
    assert_eq!(timing::duration_ms(&project), 6000);
    let filter = filter_graph::build(&project, &[true, true], None).unwrap();
    assert!(filter.contains("split=3[bg0][bg1][bg2]"));
    assert!(filter.contains(
        "trim=duration=1.000,setpts=PTS-STARTPTS,setsar=1,fps=30,settb=AVTB,format=yuv420p[hole1]"
    ));
}

#[test]
fn a_transition_consumes_its_overlap_in_both_video_and_audio() {
    let mut project = project();
    let first = project.video_track.clips[0].id.clone();
    project.video_track.clips[1].timeline_start_ms = 1500;
    project.transitions.push(dissolve(&first, 500));
    validate_project(&project, "revision").unwrap();
    assert_eq!(timing::duration_ms(&project), 4500);
    let filter = filter_graph::build(&project, &[true, true], None).unwrap();
    assert!(filter.contains("xfade=transition=fade:duration=0.500:offset=1.500"));
    assert!(filter.contains("afade=t=out:st=1.500:d=0.500"));
    assert!(filter.contains("afade=t=in:st=0:d=0.500"));
}

#[test]
fn rejects_overlaps_that_no_transition_explains() {
    let mut project = project();
    project.video_track.clips[1].timeline_start_ms = 1500;
    assert!(validate_project(&project, "revision").is_err());
}

#[test]
fn rejects_unknown_assets_and_impossible_overlaps() {
    let mut foreign = project();
    foreign.video_track.clips[0].asset_id = "foreign".into();
    assert!(validate_project(&foreign, "revision").is_err());
    let mut crowded = project();
    let first = crowded.video_track.clips[0].id.clone();
    crowded.transitions.push(dissolve(&first, 2000));
    assert!(validate_project(&crowded, "revision").is_err());
}

#[test]
fn rejects_obsolete_and_incomplete_project_shapes() {
    let version_one = r#"{"version":1,"sourceRevision":"revision","assets":[],"output":{"width":2,"height":2,"frameRate":30},"videoTrack":{"clips":[]},"audioTrack":{"clips":[]},"textTrack":{"items":[]}}"#;
    assert!(serde_json::from_str::<VideoEditProject>(version_one).is_err());
    let mut encoded = serde_json::to_value(project()).unwrap();
    encoded["videoTrack"]["clips"][0]
        .as_object_mut()
        .unwrap()
        .remove("timelineStartMs");
    assert!(serde_json::from_value::<VideoEditProject>(encoded).is_err());
    let mut encoded = serde_json::to_value(project()).unwrap();
    encoded.as_object_mut().unwrap().remove("transitions");
    assert!(serde_json::from_value::<VideoEditProject>(encoded).is_err());
}

#[test]
fn saved_snapshot_compares_typed_values_without_losing_edit_conflict_detection() {
    let requested = project();
    let mut data = serde_json::json!({"videoEditProject": requested});
    data["videoEditProject"]["videoTrack"]["clips"][0]["volume"] = serde_json::json!(1);
    data["videoEditProject"]["videoTrack"]["clips"][0]["transform"]["scale"] = serde_json::json!(1);
    assert!(super::start::validate_saved_project(&data, &requested).is_ok());
    data["videoEditProject"]["videoTrack"]["clips"][0]["volume"] = serde_json::json!(0.75);
    assert!(super::start::validate_saved_project(&data, &requested).is_err());
    data["videoEditProject"]["transitions"] = serde_json::json!([{"id":"9f3a1f3c-54f0-4c2f-9a1e-6f5c9a2b7d01","fromClipId":"d51a4384-70ab-4b44-90cc-3bfcf0f5341f","kind":"obsolete","durationMs":500}]);
    assert!(super::start::validate_saved_project(&data, &requested).is_err());
}

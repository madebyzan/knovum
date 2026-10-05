use super::contracts::{
    TextOverlay, TextPlacement, TextPreset, TextStyle, TextTrack, VideoClip, VideoTrack,
    VideoTransform,
};
use super::{filter_graph, VideoEditProject};

fn project() -> VideoEditProject {
    VideoEditProject {
        version: 1,
        source_revision: "sha256:source".into(),
        source_width: 1080,
        source_height: 1920,
        video_track: VideoTrack {
            clips: vec![
                VideoClip {
                    id: "a".into(),
                    start_ms: 1_000,
                    end_ms: 3_000,
                    volume: 1.0,
                },
                VideoClip {
                    id: "b".into(),
                    start_ms: 5_000,
                    end_ms: 8_000,
                    volume: 0.5,
                },
            ],
        },
        text_track: TextTrack {
            items: vec![TextOverlay {
                id: "t".into(),
                start_ms: 500,
                end_ms: 2_000,
                text: "Title".into(),
                placement: TextPlacement::Bottom,
                style: TextStyle {
                    family: crate::system_font::SYSTEM_SANS_FAMILY.into(),
                    weight: 400,
                    size: 30,
                    color: "#FFFFFF".into(),
                    preset: TextPreset::Outline,
                },
            }],
        },
        transform: VideoTransform {
            rotation: 90,
            flip_horizontal: true,
            flip_vertical: false,
        },
    }
}

#[test]
fn builds_trim_concat_transform_and_text_filter() {
    let filter = filter_graph::build(&project(), true, Some("captions.ass")).expect("filter graph");
    assert!(filter.contains("trim=start=1.000:end=3.000"));
    assert!(filter.contains("concat=n=2:v=1:a=1"));
    assert!(filter.contains("transpose=clock,hflip"));
    assert!(filter.contains("ass=filename=captions.ass"));
}

#[test]
fn omits_audio_graph_when_source_has_no_audio() {
    let filter =
        filter_graph::build(&project(), false, Some("captions.ass")).expect("filter graph");
    assert!(filter.contains("concat=n=2:v=1:a=0"));
    assert!(!filter.contains("atrim="));
}

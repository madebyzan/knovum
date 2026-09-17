use super::*;

#[test]
fn sanitizes_precise_cues_without_inventing_timing() {
    let cues = vec![
        SubtitleCue {
            start_ms: 100,
            end_ms: 50,
            text: "invalid".into(),
        },
        SubtitleCue {
            start_ms: 200,
            end_ms: 800,
            text: "  first  ".into(),
        },
        SubtitleCue {
            start_ms: 4_500,
            end_ms: 6_000,
            text: "last".into(),
        },
    ];
    let sanitized = sanitize(cues, 5_000);
    assert_eq!(sanitized.len(), 2);
    assert_eq!(sanitized[0].text, "first");
    assert_eq!(sanitized[1].end_ms, 5_000);
    assert!(sanitize(Vec::new(), 5_000).is_empty());
}

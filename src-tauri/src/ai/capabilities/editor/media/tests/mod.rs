use super::{
    directives::{collect_directives, EditorMediaKind, MAX_GENERATED_MEDIA_PER_TURN},
    EditorMediaCapabilities,
};

#[test]
fn parses_multimodal_image_and_video_directives_deduplicates_and_ignores_code() {
    let md = concat!(
        "![cover](knovum-gen://image?prompt=little%20fox%20with%20lantern&aspect=3:2)\n\n",
        "![dup](knovum-gen://image?prompt=little+fox+with+lantern&aspect=3:2)\n\n",
        "![scene2](knovum-gen://image?prompt=starry+sky&aspect=invalid)\n\n",
        "![scene3](knovum-gen://image?prompt=forest+stream&aspect=4:3)\n\n",
        "![scene4](knovum-gen://image?prompt=mountain+dawn&aspect=1:1)\n\n",
        "![clip](knovum-gen://video?prompt=lantern+floating+in+night+wind&aspect=16:9&duration=7)\n\n",
        "![clip-clamped](knovum-gen://video?prompt=starlight+ripple&duration=99)\n\n",
        "![unknown](knovum-gen://audio?prompt=ignored)\n\n",
        "```md\n![code](knovum-gen://image?prompt=ignored)\n```\n",
    );
    let directives = collect_directives(md);
    assert_eq!(directives.len(), 6);
    assert_eq!(directives[0].kind, EditorMediaKind::Image);
    assert_eq!(directives[0].prompt, "little fox with lantern");
    assert_eq!(directives[0].aspect, "3:2");
    assert_eq!(directives[1].aspect, "16:9");
    assert_eq!(directives[2].aspect, "4:3");
    assert_eq!(directives[3].aspect, "1:1");
    assert_eq!(directives[4].kind, EditorMediaKind::Video);
    assert_eq!(directives[4].duration_seconds, 7);
    assert_eq!(directives[5].kind, EditorMediaKind::Video);
    assert_eq!(directives[5].duration_seconds, 10);
}

#[test]
fn enforces_safety_ceiling_without_artificial_three_item_cap() {
    let md = (0..20)
        .map(|index| format!("![img-{index}](knovum-gen://image?prompt=scene+{index})"))
        .collect::<Vec<_>>()
        .join("\n\n");
    let directives = collect_directives(&md);
    assert_eq!(directives.len(), MAX_GENERATED_MEDIA_PER_TURN);
}

#[test]
fn prompt_guidance_advertises_only_enabled_modalities() {
    let none = EditorMediaCapabilities::default().prompt_guidance();
    assert!(!none.contains("knovum-gen://"));

    let image_only = EditorMediaCapabilities {
        image: true,
        video: false,
    }
    .prompt_guidance();
    assert!(image_only.contains("knovum-gen://image"));
    assert!(!image_only.contains("knovum-gen://video"));

    let both = EditorMediaCapabilities {
        image: true,
        video: true,
    }
    .prompt_guidance();
    assert!(both.contains("knovum-gen://image"));
    assert!(both.contains("knovum-gen://video"));
}

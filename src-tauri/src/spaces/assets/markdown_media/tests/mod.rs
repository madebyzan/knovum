use super::*;

#[test]
fn rewrites_inline_and_angle_media_while_ignoring_code_and_remote_urls() {
    let input = concat!(
        "# 补星星的小狐狸\n\n",
        "![小狐狸](vibe_images/fox.png)\n\n",
        "![演示视频](clips/demo.mp4)\n\n",
        "![带标题](<./a b/c.png> \"封面\")\n\n",
        "![远程](https://example.com/a.png)\n\n",
        "```markdown\n![代码里](vibe_images/ignored.png)\n```\n",
    );
    let rewritten = rewrite_markdown_media(input, |url| {
        if url.starts_with("https://") {
            Ok(None)
        } else {
            Ok(Some(format!("asset://localhost/{url}")))
        }
    })
    .unwrap();
    assert!(rewritten.contains("![小狐狸](asset://localhost/vibe_images/fox.png)"));
    assert!(rewritten.contains("![演示视频](asset://localhost/clips/demo.mp4)"));
    assert!(rewritten.contains("![带标题](asset://localhost/./a b/c.png \"封面\")"));
    assert!(rewritten.contains("![远程](https://example.com/a.png)"));
    assert!(rewritten.contains("![代码里](vibe_images/ignored.png)"));
}

#[test]
fn local_candidate_rejects_escape_and_symlinks() {
    let root = std::env::temp_dir().join(format!("knovum-md-media-{}", Uuid::new_v4()));
    let sub = root.join("vibe_images");
    fs::create_dir_all(&sub).unwrap();
    let outside = std::env::temp_dir().join(format!("knovum-outside-{}.png", Uuid::new_v4()));
    fs::write(&outside, b"outside").unwrap();
    let inside = sub.join("fox.png");
    fs::write(&inside, b"inside").unwrap();

    let canonical_root = root.canonicalize().unwrap();
    assert!(resolve_local_candidate("vibe_images/fox.png", &root, &canonical_root).is_some());
    assert!(
        resolve_local_candidate("https://example.com/fox.png", &root, &canonical_root).is_none()
    );
    assert!(
        resolve_local_candidate("asset://localhost/%2Ftmp%2Ffox.png", &root, &canonical_root)
            .is_none()
    );
    assert!(resolve_local_candidate(
        &format!("../{}", outside.file_name().unwrap().to_str().unwrap()),
        &root,
        &canonical_root
    )
    .is_none());

    let _ = fs::remove_file(&outside);
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn detects_video_containers_and_rejects_invalid_bytes() {
    let mp4 = b"\x00\x00\x00\x18ftypisom\x00\x00\x02\x00";
    let mov = b"\x00\x00\x00\x14ftypqt  \x00\x00\x02\x00";
    let webm = b"\x1A\x45\xDF\xA3\x9F\x42\x86\x81\x01";
    assert_eq!(detect_video_extension(mp4).unwrap(), "mp4");
    assert_eq!(detect_video_extension(mov).unwrap(), "mov");
    assert_eq!(detect_video_extension(webm).unwrap(), "webm");
    assert!(detect_video_extension(b"").is_err());
    assert!(detect_video_extension(b"not a video").is_err());
    assert!(is_video_path(Path::new("clip.MP4")));
    assert!(is_video_path(Path::new("clip.webm")));
    assert!(!is_video_path(Path::new("cover.png")));
}

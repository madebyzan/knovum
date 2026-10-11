use std::path::Path;

use super::policy::renderable;

#[test]
fn owned_thumbnail_claims_final_and_partial_before_writing() {
    let target = Path::new("/managed/preview.png");
    let mut claimed = Vec::new();
    let partial = super::claim_output_paths(target, &mut |path| {
        claimed.push(path.to_path_buf());
        Ok(())
    })
    .unwrap();
    assert_eq!(partial, Path::new("/managed/preview.partial"));
    assert_eq!(claimed, vec![target.to_path_buf(), partial.clone()],);
}

#[test]
fn rejected_partial_claim_stops_before_system_writer_starts() {
    let target = Path::new("/managed/preview.png");
    let claim = super::claim_output_paths(target, &mut |path| {
        if path.extension().and_then(|value| value.to_str()) == Some("partial") {
            Err("owner cancelled".into())
        } else {
            Ok(())
        }
    });
    assert_eq!(claim.unwrap_err(), "owner cancelled");
}

#[test]
fn documents_are_drawn() {
    assert!(renderable(Path::new("Quarterly.docx")));
    assert!(renderable(Path::new("Data.csv")));
}

#[test]
fn system_visual_formats_are_drawn() {
    for name in ["photo.png", "recording.mp4", "interview.mp3"] {
        assert!(
            renderable(Path::new(name)),
            "{name} must use the system thumbnail provider"
        );
    }
}

#[test]
fn formats_with_product_owned_presentations_are_skipped() {
    for name in ["main.rs", "sources.zip"] {
        assert!(!renderable(Path::new(name)), "{name} must not be redrawn");
    }
}

#[test]
fn credentials_are_never_rendered_into_app_data() {
    let names = [
        ".env",
        ".ENV.production",
        ".npmrc",
        "credentials",
        "id_rsa",
        "server.pem",
        "signing.key",
        "wallet.kdbx",
    ];
    for name in names {
        assert!(!renderable(Path::new(name)), "{name} must stay undrawn");
    }
}

/// Needs a real document and a running Quick Look daemon, so it skips itself unless the
/// environment names one: `KNOVUM_QUICKLOOK_SMOKE=/path/to/report.docx pnpm check:rust`.
#[cfg(target_os = "macos")]
#[test]
fn smoke_renders_the_document_named_by_the_environment() {
    let Some(source) = std::env::var_os("KNOVUM_QUICKLOOK_SMOKE") else {
        return;
    };
    let source = std::path::PathBuf::from(source);
    let target = std::env::temp_dir().join("knovum-quicklook-smoke.png");
    let cancellation = tokio_util::sync::CancellationToken::new();
    super::macos::save(&source, &target, &cancellation, &mut |_| Ok(()))
        .expect("Quick Look drew nothing");
    let (width, height) = ::image::image_dimensions(&target).expect("unreadable PNG");
    assert!(width > 0 && height > 0);
    println!("{} rendered as {width}x{height}", source.display());
    std::fs::remove_file(&target).ok();
}

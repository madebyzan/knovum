use std::{
    path::{Path, PathBuf},
    process::Command,
};

#[test]
fn provisioned_local_resources_starts() {
    if std::env::var("KNOVUM_BUNDLED_RUNTIME_SMOKE")
        .ok()
        .as_deref()
        != Some("1")
    {
        eprintln!(
            "bundled runtime smoke skipped; set KNOVUM_BUNDLED_RUNTIME_SMOKE=1 in a provisioned release environment"
        );
        return;
    }

    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("resources")
        .join("bundled-runtime")
        .join(target_directory());
    assert!(
        root.is_dir(),
        "bundled runtime target directory is missing: {}",
        root.display()
    );

    let ffmpeg = root.join(executable("ffmpeg"));
    assert_command_starts(&ffmpeg, &["-version"]);
    assert_command_contains(
        &root.join(executable("yt-dlp")),
        &["--version"],
        "2026.08.19",
    );
    assert_command_contains(&root.join(executable("deno")), &["--version"], "deno 2.9.6");
    assert_ffmpeg_subtitle_burn_in(&ffmpeg);
    assert_bundled_subtitle_font(&root);
    let whisper = root.join(executable("whisper-cli"));
    assert_command_starts(&whisper, &["--help"]);
    assert_whisper_capabilities(&whisper);
}

fn assert_ffmpeg_subtitle_burn_in(path: &Path) {
    let filters = command_text(path, &["-hide_banner", "-filters"]);
    assert!(
        filters.contains(" subtitles "),
        "FFmpeg runtime must include the libass subtitles filter"
    );
    let encoders = command_text(path, &["-hide_banner", "-encoders"]);
    assert!(
        encoders.contains(if cfg!(target_os = "macos") {
            " h264_videotoolbox "
        } else {
            " h264_mf "
        }),
        "FFmpeg runtime must include the platform H.264 encoder"
    );
    let configuration = command_text(path, &["-hide_banner", "-buildconf"]);
    assert!(
        !configuration.contains("--enable-nonfree"),
        "FFmpeg release runtime must remain redistributable"
    );
}

fn assert_bundled_subtitle_font(target_root: &Path) {
    let font = target_root
        .parent()
        .expect("bundled runtime target must have a shared root")
        .join("fonts")
        .join("NotoSansCJKsc-Regular.otf");
    assert!(font.is_file(), "bundled CJK subtitle font is missing");
}

fn command_text(path: &Path, args: &[&str]) -> String {
    let output = Command::new(path)
        .args(args)
        .output()
        .expect("bundled runtime capability query should run");
    format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn assert_whisper_capabilities(path: &Path) {
    let output = Command::new(path)
        .arg("--help")
        .output()
        .expect("whisper help should run");
    let help = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        help.contains("--vad"),
        "whisper runtime must include built-in VAD"
    );
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    assert_macos_framework(path, "Metal.framework");
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    assert_macos_intel_backend(path);
}

#[cfg(target_os = "macos")]
fn macos_libraries(path: &Path) -> String {
    let output = Command::new("otool")
        .args(["-L"])
        .arg(path)
        .output()
        .expect("otool should inspect whisper runtime");
    String::from_utf8_lossy(&output.stdout).into_owned()
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn assert_macos_framework(path: &Path, framework: &str) {
    let libraries = macos_libraries(path);
    assert!(
        libraries.contains(framework),
        "macOS whisper runtime must link {framework}"
    );
}

#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
fn assert_macos_intel_backend(path: &Path) {
    let libraries = macos_libraries(path);
    assert!(
        libraries.contains("Accelerate.framework"),
        "Intel macOS whisper runtime must link Accelerate.framework"
    );
    assert!(
        !libraries.contains("Metal.framework"),
        "Intel macOS whisper runtime must not link the unstable Metal backend"
    );
}

fn assert_command_starts(path: &Path, args: &[&str]) {
    assert!(
        path.is_file(),
        "bundled runtime executable is missing: {}",
        path.display()
    );
    let output = Command::new(path)
        .args(args)
        .output()
        .unwrap_or_else(|error| panic!("failed to start {}: {error}", path.display()));
    assert!(
        output.status.success(),
        "{} {:?} exited with {}\nstdout:\n{}\nstderr:\n{}",
        path.display(),
        args,
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}

fn assert_command_contains(path: &Path, args: &[&str], expected: &str) {
    assert!(
        path.is_file(),
        "bundled runtime executable is missing: {}",
        path.display()
    );
    let output = Command::new(path)
        .args(args)
        .output()
        .unwrap_or_else(|error| panic!("failed to start {}: {error}", path.display()));
    let text = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.status.success() && text.contains(expected),
        "{} {:?} must report {expected}",
        path.display(),
        args
    );
}

const fn target_directory() -> &'static str {
    if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        "macos-arm64"
    } else if cfg!(all(target_os = "macos", target_arch = "x86_64")) {
        "macos-x64"
    } else if cfg!(all(target_os = "windows", target_arch = "aarch64")) {
        "windows-arm64"
    } else if cfg!(all(target_os = "windows", target_arch = "x86_64")) {
        "windows-x64"
    } else if cfg!(all(target_os = "linux", target_arch = "aarch64")) {
        "linux-arm64"
    } else if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        "linux-x64"
    } else {
        "unsupported"
    }
}

fn executable(base: &str) -> String {
    if cfg!(target_os = "windows") {
        format!("{base}.exe")
    } else {
        base.to_owned()
    }
}

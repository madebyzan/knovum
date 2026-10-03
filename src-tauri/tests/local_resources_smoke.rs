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
    assert_ffmpeg_subtitle_burn_in(&ffmpeg);
    let python = root
        .join("python-runtime")
        .join(if cfg!(target_os = "windows") {
            "python.exe"
        } else {
            "bin/python3"
        });
    assert_command_starts(&python, &["--version"]);
    let packages = root.join("python-runtime").join("packages");
    let packages = packages
        .to_str()
        .expect("Python package path must be UTF-8");
    let backend = if cfg!(target_os = "macos") {
        "mlx_whisper"
    } else {
        "faster_whisper"
    };
    assert_command_starts(
        &python,
        &[
            "-I",
            "-c",
            "import sys; sys.path.insert(0, sys.argv[1]); import knovum_inference.worker, onnxruntime, torch, torchcodec; __import__(sys.argv[2])",
            packages,
            backend,
        ],
    );
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

const fn target_directory() -> &'static str {
    if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        "macos-arm64"
    } else if cfg!(all(target_os = "windows", target_arch = "x86_64")) {
        "windows-x64"
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

use std::{
    path::{Path, PathBuf},
    process::Command,
};

#[test]
#[ignore = "requires provisioned compiled runtime artifacts"]
fn provisioned_local_resources_starts() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("resources")
        .join("bundled-runtime")
        .join(target_directory());
    assert!(
        root.is_dir(),
        "bundled runtime target directory is missing: {}",
        root.display()
    );

    let ffmpeg = root.join(if cfg!(target_os = "windows") {
        "Library/bin/ffmpeg.exe"
    } else {
        "bin/ffmpeg"
    });
    assert_command_starts(&ffmpeg, &["-version"]);
    assert_ffmpeg_subtitle_burn_in(&ffmpeg);
    let worker = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../python/src");
    let worker = worker.to_str().expect("Worker path must be UTF-8");
    for (profile, module) in [
        ("resources", "knovum_model_resources.server"),
        ("light-inference", "knovum_inference.server"),
        ("torch-inference", "knovum_inference.server"),
        ("voice-generation", "knovum_inference.server"),
        ("agent", "knovum_agent_runtime.server"),
    ] {
        let runtime = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../runtime-artifacts")
            .join(profile)
            .join(target_directory())
            .join("runtime");
        let python = runtime.join(if cfg!(target_os = "windows") {
            "python.exe"
        } else {
            "bin/python3"
        });
        let packages = runtime.join("packages");
        assert_command_starts(
            &python,
            &[
                "-I", "-B", "-X", "utf8", "-c",
                "import os,sys;dll=os.add_dll_directory(os.path.join(sys.prefix,'Library','bin')) if os.name=='nt' else None;sys.path[:0]=[sys.argv[1],sys.argv[2]];assert sys.version_info[:3]==(3,11,17);__import__(sys.argv[3])",
                worker,
                packages.to_str().expect("Python package path must be UTF-8"),
                module,
            ],
        );
    }
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
    } else if cfg!(all(target_os = "macos", target_arch = "x86_64")) {
        "macos-x64"
    } else if cfg!(all(target_os = "windows", target_arch = "x86_64")) {
        "windows-x64"
    } else {
        "unsupported"
    }
}

use std::{path::Path, process::Command};

// Cargo owns OUT_DIR; CMake owns supplier dependency tracking and installation.
pub fn install(target: &str, platform: &str, root: &Path) {
    if root.exists() {
        std::fs::remove_dir_all(root).expect("CMake install tree must be replaceable");
    }
    let build = root.parent().unwrap().join("build");
    let mut configure = Command::new("cmake");
    configure
        // Archived suppliers must not discover the surrounding application's Git repository.
        .env("GIT_CEILING_DIRECTORIES", &build)
        .args(["-S", "cmake", "-B"])
        .arg(&build)
        .arg(format!("-DKNOVUM_TARGET={platform}"))
        .arg(format!("-DCMAKE_INSTALL_PREFIX={}", root.display()))
        .args([
            "-DCMAKE_BUILD_TYPE=Release",
            "-DCMAKE_POSITION_INDEPENDENT_CODE=ON",
        ]);
    if target.contains("apple") {
        configure.arg("-DCMAKE_OSX_DEPLOYMENT_TARGET=14.0");
        configure.arg(if target.starts_with("aarch64") {
            "-DCMAKE_OSX_ARCHITECTURES=arm64"
        } else {
            "-DCMAKE_OSX_ARCHITECTURES=x86_64"
        });
    }
    assert!(
        configure.status().expect("CMake is required").success(),
        "CMake configuration failed"
    );
    assert!(
        Command::new("cmake")
            .env("GIT_CEILING_DIRECTORIES", &build)
            .arg("--build")
            .arg(build)
            .args([
                "--target",
                "install",
                "--config",
                "Release",
                "--parallel",
                "4"
            ])
            .status()
            .expect("CMake is required")
            .success(),
        "CMake build failed"
    );
}

use std::{
    env,
    path::{Path, PathBuf},
    process::Command,
};

// MSVC and MSBuild write intermediate files (object databases, tracker logs, try-compile scratch
// trees) through MAX_PATH-limited interfaces, and the suppliers nest roughly 175 further
// characters of their own ExternalProject and compiler-detection directories below this build
// tree. Cargo's hashed OUT_DIR alone is already longer than 100 characters, so Windows supplier
// builds own a short deterministic build tree outside it. The standard %LOCALAPPDATA% stays
// within this budget for every Windows account name, which SAM limits to 20 characters.
const WINDOWS_BUILD_TREE_BUDGET: usize = 72;

// Cargo owns OUT_DIR; CMake owns supplier dependency tracking and installation.
pub fn install(target: &str, platform: &str, root: &Path) {
    if root.exists() {
        std::fs::remove_dir_all(root).expect("CMake install tree must be replaceable");
    }
    let out = root.parent().expect("OUT_DIR owns the install tree");
    let build = if target.contains("windows") {
        windows_build_tree(&windows_build_cache(), out)
    } else {
        out.join("build")
    };
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

fn windows_build_cache() -> PathBuf {
    PathBuf::from(
        env::var_os("LOCALAPPDATA")
            .expect("Windows supplier builds need LOCALAPPDATA for a short build tree"),
    )
}

/// Build tree for one OUT_DIR, short enough for the suppliers' nested MSVC tool paths.
pub fn windows_build_tree(cache: &Path, out: &Path) -> PathBuf {
    let tree = cache
        .join("knovum-build")
        .join(format!("{:016x}", fingerprint(out)));
    assert!(
        tree.as_os_str().len() <= WINDOWS_BUILD_TREE_BUDGET,
        "Windows supplier builds need a build tree within {WINDOWS_BUILD_TREE_BUDGET} characters: {}",
        tree.display()
    );
    tree
}

/// FNV-1a keeps one supplier build tree per OUT_DIR across runs without a dependency.
fn fingerprint(path: &Path) -> u64 {
    path.to_string_lossy()
        .bytes()
        .fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
            (hash ^ u64::from(byte)).wrapping_mul(0x0000_0100_0000_01b3)
        })
}

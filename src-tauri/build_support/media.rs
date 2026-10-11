pub(crate) fn configure() {
    let directory = std::env::var("DEP_KNOVUM_MEDIA_LIBRARIES_DIRECTORY")
        .expect("Cargo must prepare qualified FFmpeg");
    println!("cargo:rustc-env=KNOVUM_MEDIA_RUNTIME_DIRECTORY={directory}");
}

// knovum_lib's build script (tauri-build, static VC runtime) writes an empty `msvcrt.lib` into its
// OUT_DIR and puts that directory on the propagated native link-search path. The `knovum` package
// cancels it with `/NODEFAULTLIB:msvcrt.lib` plus a static CRT, but `cargo:rustc-link-arg` does not
// reach this separate workspace package, so its `/defaultlib:msvcrt` would resolve to the empty stub
// (LNK4003) and leave `__CxxFrameHandler3` unresolved (LNK2001). Mirror the same CRT selection.
fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    if std::env::var("CARGO_CFG_TARGET_ENV").as_deref() != Ok("msvc") {
        return;
    }
    for conflicting in [
        "libvcruntimed.lib",
        "vcruntime.lib",
        "vcruntimed.lib",
        "libcmtd.lib",
        "msvcrt.lib",
        "msvcrtd.lib",
        "libucrt.lib",
        "libucrtd.lib",
    ] {
        println!("cargo:rustc-link-arg=/NODEFAULTLIB:{conflicting}");
    }
    for selected in ["libcmt.lib", "libvcruntime.lib", "ucrt.lib"] {
        println!("cargo:rustc-link-arg=/DEFAULTLIB:{selected}");
    }
}

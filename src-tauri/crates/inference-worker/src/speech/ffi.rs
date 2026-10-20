use crate::initialization::VerifiedRuntime;
use std::{ffi::CString, path::PathBuf};

#[allow(
    non_snake_case,
    non_camel_case_types,
    non_upper_case_globals,
    dead_code,
    clippy::all
)]
pub(super) mod sherpa {
    include!(concat!(env!("OUT_DIR"), "/SherpaApi.rs"));
}
#[allow(
    non_snake_case,
    non_camel_case_types,
    non_upper_case_globals,
    dead_code,
    clippy::all
)]
pub(super) mod whisper {
    include!(concat!(env!("OUT_DIR"), "/WhisperApi.rs"));
    include!(concat!(env!("OUT_DIR"), "/GgmlApi.rs"));
}
#[allow(
    non_snake_case,
    non_camel_case_types,
    non_upper_case_globals,
    dead_code,
    clippy::all
)]
pub(super) mod voice {
    include!(concat!(env!("OUT_DIR"), "/VoiceApi.rs"));
}
#[allow(
    non_snake_case,
    non_camel_case_types,
    non_upper_case_globals,
    dead_code,
    clippy::all
)]
pub(super) mod audio {
    include!(concat!(env!("OUT_DIR"), "/AudioApi.rs"));
}

/// Resolves a pinned speech library inside the runtime this worker verified.
///
/// Library naming belongs here; inventory membership belongs to [`VerifiedRuntime`], which holds
/// the manifest this process hashed. Nothing re-reads the manifest from disk after verification.
pub(super) fn library(runtime: &VerifiedRuntime, name: &str) -> Result<PathBuf, String> {
    let extension = if cfg!(windows) { "dll" } else { "dylib" };
    let relative = if name == "whisper" {
        format!(
            "compiled-libraries/whisper/{}{extension}",
            if cfg!(windows) {
                "whisper."
            } else {
                "libwhisper."
            }
        )
    } else {
        format!("compiled-libraries/{name}.{extension}")
    };
    runtime.member(&relative)
}

/// Loads the pinned Sherpa C API from the verified runtime.
///
/// Both Sherpa capabilities load the same pinned library, so its supplier version is established
/// here once instead of in whichever capability remembered to check it.
pub(super) fn sherpa_api(runtime: &VerifiedRuntime) -> Result<sherpa::SherpaApi, String> {
    let api =
        unsafe { sherpa::SherpaApi::from_library(load(&library(runtime, "sherpa-onnx-c-api")?)?) }
            .map_err(|error| format!("runtime_mismatch: {error}"))?;
    let version = unsafe { std::ffi::CStr::from_ptr(api.SherpaOnnxGetVersionStr()) }
        .to_str()
        .map_err(|error| error.to_string())?;
    if version != "1.13.8" {
        return Err("runtime_mismatch: Sherpa version differs".into());
    }
    Ok(api)
}

pub(super) fn load(path: &std::path::Path) -> Result<libloading::Library, String> {
    // Callers supply only a library in the hash-verified, immutable inference runtime inventory.
    #[cfg(windows)]
    windows_search_scope(path)?;
    #[cfg(windows)]
    let library = unsafe {
        libloading::os::windows::Library::load_with_flags(
            path,
            0x00000100 | 0x00000800 | 0x00000400,
        )
    }
    .map(Into::into);
    #[cfg(not(windows))]
    let library = unsafe { libloading::Library::new(path) };
    library.map_err(|error| format!("runtime_mismatch: {error}"))
}

pub(super) fn string(value: &str) -> Result<CString, String> {
    CString::new(value).map_err(|_| "invalid_input: C string contains a zero byte".into())
}

#[cfg(windows)]
fn windows_search_scope(path: &std::path::Path) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or("runtime_mismatch: library parent is missing")?;
    let root = if parent.file_name() == Some(std::ffi::OsStr::new("whisper")) {
        parent.parent().and_then(std::path::Path::parent)
    } else {
        parent.parent()
    }
    .ok_or("runtime_mismatch: library runtime root is missing")?;
    prepare_search_directory(root)
}

#[cfg(windows)]
pub(crate) fn prepare_search_directory(root: &std::path::Path) -> Result<(), String> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::System::LibraryLoader::{
        AddDllDirectory, SetDefaultDllDirectories, LOAD_LIBRARY_SEARCH_SYSTEM32,
        LOAD_LIBRARY_SEARCH_USER_DIRS,
    };
    static SCOPE: std::sync::OnceLock<Result<(), String>> = std::sync::OnceLock::new();
    SCOPE
        .get_or_init(|| {
            if unsafe {
                SetDefaultDllDirectories(
                    LOAD_LIBRARY_SEARCH_SYSTEM32 | LOAD_LIBRARY_SEARCH_USER_DIRS,
                )
            } == 0
            {
                return Err(std::io::Error::last_os_error().to_string());
            }
            // Each worker owns one immutable runtime. OS directory cookies live until process exit.
            for directory in [root.to_path_buf(), root.join("bin")] {
                let wide: Vec<u16> = directory.as_os_str().encode_wide().chain(Some(0)).collect();
                if unsafe { AddDllDirectory(wide.as_ptr()) }.is_null() {
                    return Err(std::io::Error::last_os_error().to_string());
                }
            }
            Ok(())
        })
        .clone()
}

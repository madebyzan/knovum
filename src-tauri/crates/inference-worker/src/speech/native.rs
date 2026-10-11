use crate::artifact;
use knovum_inference_contracts::{
    native_manifest::NativeRuntimeManifest, runtime::RuntimeInitializeParams,
};
use std::{ffi::CString, io::Read, path::PathBuf};

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

pub(super) fn path(params: &RuntimeInitializeParams, name: &str) -> Result<PathBuf, String> {
    library_path(std::path::Path::new(&params.runtime_manifest_path), name)
}

pub(super) fn library_path(manifest_path: &std::path::Path, name: &str) -> Result<PathBuf, String> {
    let mut bytes = Vec::new();
    artifact::open_regular(manifest_path)?
        .take(1_048_577)
        .read_to_end(&mut bytes)
        .map_err(|error| error.to_string())?;
    let manifest: NativeRuntimeManifest =
        serde_json::from_slice(&bytes).map_err(|error| format!("runtime_mismatch: {error}"))?;
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
    if !manifest.files.iter().any(|file| file.path == relative) {
        return Err(
            "runtime_mismatch: native speech library is absent from the verified runtime".into(),
        );
    }
    Ok(manifest_path
        .parent()
        .ok_or("runtime_mismatch: native runtime directory is missing")?
        .join(relative))
}

pub(super) fn load(path: &std::path::Path) -> Result<libloading::Library, String> {
    // Callers supply only a library in the hash-verified, immutable native runtime inventory.
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
    CString::new(value).map_err(|_| "invalid_input: native string contains a zero byte".into())
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

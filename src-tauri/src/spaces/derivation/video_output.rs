use std::{fs, io::Read, path::PathBuf};

use sha2::{Digest, Sha256};

pub struct RenderedVideo {
    pub path: String,
    pub size: i64,
    pub revision: String,
}

pub fn inspect(path: PathBuf) -> Result<RenderedVideo, String> {
    let metadata = fs::metadata(&path).map_err(|error| error.to_string())?;
    if metadata.len() == 0 {
        return Err("FFmpeg created an empty video".into());
    }
    let mut file = fs::File::open(&path).map_err(|error| error.to_string())?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 65_536];
    loop {
        let read = file.read(&mut buffer).map_err(|error| error.to_string())?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(RenderedVideo {
        path: path.to_string_lossy().into_owned(),
        size: metadata.len() as i64,
        revision: format!("sha256:{:x}", hasher.finalize()),
    })
}

pub const fn platform_h264_encoder() -> Result<&'static str, &'static str> {
    #[cfg(target_os = "macos")]
    return Ok("h264_videotoolbox");
    #[cfg(target_os = "windows")]
    return Ok("h264_mf");
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    Err("Video rendering is supported only on macOS and Windows")
}

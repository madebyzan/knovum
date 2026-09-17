use std::{fs::File, io::Read, path::Path};

pub(super) fn validate_container(path: &Path, mime_type: &str) -> Result<(), String> {
    let mut file = File::open(path).map_err(|error| error.to_string())?;
    let mut prefix = [0_u8; 16 * 1024];
    let read = file.read(&mut prefix).map_err(|error| error.to_string())?;
    let detected = infer::get(&prefix[..read])
        .map(|kind| kind.mime_type())
        .ok_or_else(|| "Audio recording container could not be verified".to_owned())?;
    let matches = match normalized_mime(mime_type) {
        "audio/webm" => detected.ends_with("/webm"),
        "audio/mp4" => detected.contains("mp4") || detected.contains("m4a"),
        "audio/ogg" => detected.contains("ogg"),
        _ => false,
    };
    matches
        .then_some(())
        .ok_or_else(|| "Audio recording container does not match the recorder format".to_owned())
}

pub(super) fn extension_for_mime(mime_type: &str) -> Result<&'static str, String> {
    match normalized_mime(mime_type) {
        "audio/webm" => Ok("weba"),
        "audio/mp4" => Ok("m4a"),
        "audio/ogg" => Ok("oga"),
        _ => Err("Unsupported audio recording format".to_owned()),
    }
}

fn normalized_mime(mime_type: &str) -> &str {
    mime_type.split(';').next().unwrap_or("").trim()
}

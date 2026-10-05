use super::NormalizedSource;
use crate::spaces::speech::project_config::AudiobookOutputConfig;
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

pub(in crate::spaces::speech::project_publication) fn write_chapter_metadata(
    directory: &Path,
    config: &AudiobookOutputConfig,
    chapter_keys: &[Option<String>],
    sources: &[NormalizedSource],
    sample_rate: u32,
) -> Result<PathBuf, String> {
    let path = directory.join("chapters.ffmetadata.part");
    if path.exists() {
        fs::remove_file(&path).map_err(|error| error.to_string())?;
    }
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .map_err(|error| error.to_string())?;
    file.write_all(b";FFMETADATA1\n")
        .map_err(|error| error.to_string())?;
    for (key, value) in [
        ("title", &config.metadata.title),
        ("artist", &config.metadata.narrator),
        ("album_artist", &config.metadata.author),
        ("composer", &config.metadata.author),
    ] {
        writeln!(file, "{key}={}", ffmetadata_value(value)).map_err(|error| error.to_string())?;
    }
    let mut end_samples = 0_u64;
    let mut source_index = 0_usize;
    for chapter in &config.chapters {
        let start_samples = end_samples;
        while source_index < sources.len()
            && chapter_keys[source_index].as_deref() == Some(chapter.key.as_str())
        {
            end_samples = end_samples
                .checked_add(sources[source_index].sample_count)
                .ok_or("Speech chapter sample count overflow")?;
            source_index += 1;
        }
        writeln!(
            file, "[CHAPTER]\nTIMEBASE=1/{sample_rate}\nSTART={start_samples}\nEND={end_samples}\ntitle={}",
            ffmetadata_value(&chapter.title),
        ).map_err(|error| error.to_string())?;
    }
    if source_index != sources.len() {
        return Err("Speech chapter layout does not cover every rendered segment".into());
    }
    file.sync_all().map_err(|error| error.to_string())?;
    Ok(path)
}

fn ffmetadata_value(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '\n' | '\r' => escaped.push(' '),
            '\\' | '=' | ';' | '#' => {
                escaped.push('\\');
                escaped.push(character);
            }
            _ => escaped.push(character),
        }
    }
    escaped
}

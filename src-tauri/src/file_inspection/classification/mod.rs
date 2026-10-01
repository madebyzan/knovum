mod languages;
mod sensitive;

use std::path::Path;

pub use sensitive::is_sensitive_file;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResourceClassification {
    SourceCode(&'static str),
    TextDocument,
    Image,
    Video,
    Audio,
    Archive,
    File,
}

impl ResourceClassification {
    pub const fn node_kind(self) -> &'static str {
        match self {
            Self::SourceCode(_) => "code",
            Self::Image => "image",
            Self::Video => "video",
            Self::Audio => "audio",
            Self::TextDocument | Self::Archive | Self::File => "file",
        }
    }

    pub const fn source_language(self) -> Option<&'static str> {
        match self {
            Self::SourceCode(language) => Some(language),
            _ => None,
        }
    }

    pub const fn is_text_document(self) -> bool {
        matches!(self, Self::TextDocument)
    }
}

pub fn classify_file(path: &Path) -> ResourceClassification {
    if is_archive(path) {
        return ResourceClassification::Archive;
    }
    if is_sensitive_file(path) {
        return ResourceClassification::File;
    }
    if let Some(language) = languages::source_language(path) {
        return ResourceClassification::SourceCode(language);
    }
    if is_text_document(path) {
        return ResourceClassification::TextDocument;
    }
    classify_media(path)
}

fn is_archive(path: &Path) -> bool {
    let Some(name) = file_name_lower(path) else {
        return false;
    };
    [
        ".zip", ".jar", ".apk", ".ipa", ".epub", ".vsix", ".tar", ".tar.gz", ".tgz",
    ]
    .iter()
    .any(|suffix| name.ends_with(suffix))
}

fn is_text_document(path: &Path) -> bool {
    matches!(
        extension(path).as_deref(),
        Some("csv" | "tsv" | "txt" | "log" | "rst" | "adoc" | "ics" | "vcf")
    )
}

fn classify_media(path: &Path) -> ResourceClassification {
    match mime_guess::from_path(path).first() {
        Some(media_type) if media_type.type_() == mime::IMAGE => ResourceClassification::Image,
        Some(media_type) if media_type.type_() == mime::VIDEO => ResourceClassification::Video,
        Some(media_type) if media_type.type_() == mime::AUDIO => ResourceClassification::Audio,
        _ => ResourceClassification::File,
    }
}

pub(super) fn extension(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|value| value.to_str())
        .map(str::to_ascii_lowercase)
}

pub(super) fn file_name_lower(path: &Path) -> Option<String> {
    path.file_name()
        .and_then(|value| value.to_str())
        .map(str::to_ascii_lowercase)
}

#[cfg(test)]
mod tests;

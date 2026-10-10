use std::{fs::File, path::Path};

#[derive(Default)]
pub(super) struct TextBudget(usize);

impl TextBudget {
    pub(super) fn consume(&mut self, text: &str) -> Result<(), String> {
        let length = text.encode_utf16().count() + 2;
        if self.0 + length > super::document_text::MAX_TEXT_UTF16 {
            return Err("Office text exceeds the processing budget".into());
        }
        self.0 += length;
        Ok(())
    }
}

/// Admission control before the vendor parser expands an OOXML archive into its object model.
pub(super) fn preflight(source: &Path) -> Result<(), String> {
    let file = File::open(source).map_err(|error| error.to_string())?;
    if file.metadata().map_err(|error| error.to_string())?.len() > 128 * 1024 * 1024 {
        return Err("Office document exceeds 128 MiB".into());
    }
    let extension = source
        .extension()
        .and_then(|value| value.to_str())
        .map(str::to_ascii_lowercase);
    if !matches!(extension.as_deref(), Some("docx" | "xlsx" | "pptx")) {
        return Ok(());
    }
    check_archive(file)
}

pub(crate) fn preflight_archive(
    bytes: &[u8],
    format: office_oxide::DocumentFormat,
) -> Result<(), String> {
    use office_oxide::DocumentFormat;
    if matches!(
        format,
        DocumentFormat::Docx | DocumentFormat::Xlsx | DocumentFormat::Pptx
    ) {
        check_archive(std::io::Cursor::new(bytes))?;
    }
    Ok(())
}

fn check_archive(reader: impl std::io::Read + std::io::Seek) -> Result<(), String> {
    let mut archive = zip::ZipArchive::new(reader).map_err(|error| error.to_string())?;
    if archive.len() > 20_000 {
        return Err("Office archive exceeds 20000 entries".into());
    }
    let mut expanded = 0_u64;
    for index in 0..archive.len() {
        let entry = archive
            .by_index_raw(index)
            .map_err(|error| error.to_string())?;
        expanded = expanded
            .checked_add(entry.size())
            .ok_or("Office archive size overflow")?;
        if entry.size() > 128 * 1024 * 1024 || expanded > 256 * 1024 * 1024 {
            return Err("Office expanded content exceeds the processing budget".into());
        }
    }
    Ok(())
}

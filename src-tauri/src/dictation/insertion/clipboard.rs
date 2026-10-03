use clipboard_rs::{Clipboard, ClipboardContent, ClipboardContext, ContentFormat};

const MARKER_FORMAT: &str = "application/x-knovum-dictation-marker";

pub(super) struct ClipboardTransaction {
    clipboard: ClipboardContext,
    original: Option<Vec<ClipboardContent>>,
    marker: Vec<u8>,
}

impl ClipboardTransaction {
    pub(super) fn begin(text: &str) -> Result<Self, String> {
        let clipboard = ClipboardContext::new().map_err(|error| error.to_string())?;
        let mut original = clipboard
            .get(&[
                ContentFormat::Files,
                ContentFormat::Image,
                ContentFormat::Rtf,
                ContentFormat::Html,
                ContentFormat::Text,
            ])
            .map_err(|error| error.to_string())?;
        if original
            .iter()
            .any(|content| matches!(content, ClipboardContent::Files(_)))
        {
            original.retain(|content| !matches!(content, ClipboardContent::Image(_)));
        }
        let marker = uuid::Uuid::new_v4().as_bytes().to_vec();
        clipboard
            .set(vec![
                ClipboardContent::Text(text.to_owned()),
                ClipboardContent::Other(MARKER_FORMAT.into(), marker.clone()),
            ])
            .map_err(|error| error.to_string())?;
        let staged = clipboard
            .get_buffer(MARKER_FORMAT)
            .is_ok_and(|value| value == marker)
            && clipboard.get_text().is_ok_and(|value| value == text);
        if !staged {
            restore(&clipboard, original)?;
            return Err("Unable to stage dictation text on the clipboard".into());
        }
        Ok(Self {
            clipboard,
            original: Some(original),
            marker,
        })
    }

    pub(super) fn restore_if_unchanged(&mut self) {
        let marker_matches = self
            .clipboard
            .get_buffer(MARKER_FORMAT)
            .is_ok_and(|value| value == self.marker);
        if marker_matches {
            if let Some(original) = self.original.take() {
                if let Err(error) = restore(&self.clipboard, original) {
                    log::warn!("failed to restore dictation clipboard: {error}");
                }
            }
        }
    }
}

impl Drop for ClipboardTransaction {
    fn drop(&mut self) {
        self.restore_if_unchanged();
    }
}

pub(super) fn write_text(text: String) -> Result<(), String> {
    let clipboard = ClipboardContext::new().map_err(|error| error.to_string())?;
    clipboard.set_text(text).map_err(|error| error.to_string())
}

fn restore(clipboard: &ClipboardContext, contents: Vec<ClipboardContent>) -> Result<(), String> {
    if contents.is_empty() {
        clipboard.clear()
    } else {
        clipboard.set(contents)
    }
    .map_err(|error| error.to_string())
}

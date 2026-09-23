use serde::Deserialize;

use super::contracts::MediaKind;

#[derive(Debug, Deserialize)]
pub(crate) struct YtDlpMetadata {
    pub id: String,
    pub title: String,
    pub extractor_key: String,
    pub webpage_url: String,
    pub uploader: Option<String>,
    pub channel: Option<String>,
    pub duration: Option<f64>,
    pub width: Option<u64>,
    pub height: Option<u64>,
    pub filepath: String,
    pub vcodec: Option<String>,
    pub acodec: Option<String>,
}

impl YtDlpMetadata {
    pub(crate) fn media_kind(&self) -> Result<MediaKind, String> {
        match self.vcodec.as_deref() {
            Some("none") => Ok(MediaKind::Audio),
            Some(_) => Ok(MediaKind::Video),
            None => Err("yt-dlp omitted the selected video codec".into()),
        }
    }

    pub(crate) fn is_playable(&self, kind: MediaKind, extension: &str) -> Result<bool, String> {
        let audio = self
            .acodec
            .as_deref()
            .ok_or("yt-dlp omitted the selected audio codec")?;
        let aac = audio == "none" || audio == "aac" || audio.starts_with("mp4a");
        Ok(match kind {
            MediaKind::Video => {
                extension == "mp4"
                    && self
                        .vcodec
                        .as_deref()
                        .is_some_and(|codec| codec == "h264" || codec.starts_with("avc1"))
                    && aac
            }
            MediaKind::Audio => matches!(extension, "m4a" | "mp4") && audio != "none" && aac,
        })
    }

    pub(crate) fn creator(&self) -> Option<String> {
        self.uploader
            .as_deref()
            .or(self.channel.as_deref())
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_owned)
    }

    pub(crate) fn duration_seconds(&self) -> Option<u64> {
        const JS_MAX_SAFE_INTEGER: f64 = 9_007_199_254_740_991.0;
        self.duration
            .filter(|value| value.is_finite() && (0.0..=JS_MAX_SAFE_INTEGER).contains(value))
            .map(|value| value.round() as u64)
    }

    pub(crate) fn display_dimensions(&self) -> Option<(u32, u32)> {
        let width = u32::try_from(self.width?).ok()?;
        let height = u32::try_from(self.height?).ok()?;
        (width > 0 && width <= 100_000 && height > 0 && height <= 100_000)
            .then_some((width, height))
    }
}

pub(crate) fn is_http_url(value: &str) -> bool {
    reqwest::Url::parse(value).is_ok_and(|url| {
        matches!(url.scheme(), "http" | "https")
            && url.host_str().is_some()
            && url.username().is_empty()
            && url.password().is_none()
    })
}

#[cfg(test)]
mod tests;

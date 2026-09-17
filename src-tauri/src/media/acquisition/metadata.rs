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
mod tests {
    use super::{MediaKind, YtDlpMetadata};

    fn metadata(vcodec: &str, acodec: &str) -> YtDlpMetadata {
        serde_json::from_value(serde_json::json!({
            "id": "media-1",
            "title": "Video",
            "extractor_key": "Youtube",
            "webpage_url": "https://www.youtube.com/watch?v=media-1",
            "filepath": "/assets/media.mp4",
            "vcodec": vcodec,
            "acodec": acodec,
        }))
        .unwrap()
    }

    #[test]
    fn mp4_container_does_not_make_av1_playable() {
        let source = metadata("av01.0.08M.08", "mp4a.40.2");
        assert_eq!(source.media_kind().unwrap(), MediaKind::Video);
        assert!(!source.is_playable(MediaKind::Video, "mp4").unwrap());
        assert!(metadata("avc1.640028", "mp4a.40.2")
            .is_playable(MediaKind::Video, "mp4")
            .unwrap());
    }

    #[test]
    fn audio_kind_uses_the_selected_codec() {
        let source = metadata("none", "opus");
        assert_eq!(source.media_kind().unwrap(), MediaKind::Audio);
        assert!(!source.is_playable(MediaKind::Audio, "webm").unwrap());
    }
}

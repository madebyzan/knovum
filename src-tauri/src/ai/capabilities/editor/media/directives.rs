use percent_encoding::percent_decode_str;

use crate::spaces::assets::markdown_media;

const MEDIA_DIRECTIVE_SCHEME: &str = "knovum-gen://";
pub(super) const MAX_GENERATED_MEDIA_PER_TURN: usize = 16;
const MAX_PROMPT_CHARS: usize = 1_500;
const DEFAULT_VIDEO_DURATION_SECONDS: u8 = 5;
const MIN_VIDEO_DURATION_SECONDS: u8 = 4;
const MAX_VIDEO_DURATION_SECONDS: u8 = 10;

/// Modalities supported by Knovum's inline Markdown generation directive protocol
/// (`knovum-gen://<kind>?prompt=...`). Extending Knovum with future modalities (e.g. audio)
/// requires adding a variant here and a matching generation handler in `media::mod`.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum EditorMediaKind {
    Image,
    Video,
}

impl EditorMediaKind {
    pub(crate) const fn progress_label(self) -> &'static str {
        match self {
            Self::Image => "illustration",
            Self::Video => "video",
        }
    }

    fn from_host(host: &str) -> Option<Self> {
        match host {
            "image" => Some(Self::Image),
            "video" => Some(Self::Video),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct MediaGenerationDirective {
    pub(crate) kind: EditorMediaKind,
    pub(crate) prompt: String,
    pub(crate) aspect: String,
    pub(crate) duration_seconds: u8,
}

pub(crate) fn collect_directives(markdown: &str) -> Vec<MediaGenerationDirective> {
    let mut directives = Vec::new();
    let _ = markdown_media::rewrite_markdown_media(markdown, |dest_url| {
        if directives.len() < MAX_GENERATED_MEDIA_PER_TURN {
            if let Some(directive) = parse_directive(dest_url) {
                if !directives.contains(&directive) {
                    directives.push(directive);
                }
            }
        }
        Ok(None)
    });
    directives
}

pub(crate) fn parse_directive(dest_url: &str) -> Option<MediaGenerationDirective> {
    let rest = dest_url.trim().strip_prefix(MEDIA_DIRECTIVE_SCHEME)?;
    let (host, query) = rest.split_once('?')?;
    let kind = EditorMediaKind::from_host(host)?;

    let mut prompt = None;
    let mut aspect = "16:9".to_owned();
    let mut duration_seconds = DEFAULT_VIDEO_DURATION_SECONDS;

    for pair in query.split('&') {
        let Some((key, value)) = pair.split_once('=') else {
            continue;
        };
        let normalized = value.replace('+', " ");
        let decoded = percent_decode_str(&normalized).decode_utf8().ok()?;
        let trimmed = decoded.trim();
        match key {
            "prompt" if !trimmed.is_empty() => {
                prompt = Some(trimmed.chars().take(MAX_PROMPT_CHARS).collect::<String>());
            }
            "aspect" => {
                if matches!(
                    trimmed,
                    "1:1" | "3:2" | "2:3" | "4:3" | "3:4" | "16:9" | "9:16"
                ) {
                    aspect = trimmed.to_owned();
                }
            }
            "duration" => {
                if let Ok(parsed) = trimmed.parse::<u8>() {
                    duration_seconds =
                        parsed.clamp(MIN_VIDEO_DURATION_SECONDS, MAX_VIDEO_DURATION_SECONDS);
                }
            }
            _ => {}
        }
    }

    Some(MediaGenerationDirective {
        kind,
        prompt: prompt?,
        aspect,
        duration_seconds: if kind == EditorMediaKind::Video {
            duration_seconds
        } else {
            0
        },
    })
}

use super::super::ass_format;
use crate::desktop::system_font::SYSTEM_SANS_FAMILY;
use crate::media::{subtitle_timing::is_cjk, transcript::SubtitleCue};

const SCRIPT_HEIGHT: u64 = 720;
const CJK_CUE_BUDGET: usize = 20;
const LATIN_CUE_BUDGET: usize = 42;

/// Renders broadcast-grade ASS v4+ subtitles at an explicit 720p virtual script resolution so
/// typography, hairline outline, soft drop shadow, and safe margins scale cleanly on every frame
/// aspect ratio (16:9 landscape, 9:16 vertical, 1080p, 4K) without libass's 288p SRT distortion.
pub(crate) fn from_cues(
    cues: &[SubtitleCue],
    duration_ms: u64,
    frame_size: Option<(u32, u32)>,
) -> Result<String, String> {
    let (script_width, font_size) = match frame_size {
        Some((w, h)) if w > 0 && h > 0 => {
            let scaled_w = ((u64::from(w) * SCRIPT_HEIGHT) + u64::from(h) / 2) / u64::from(h);
            let size = if scaled_w < 600 { 28 } else { 34 };
            (scaled_w.max(360), size)
        }
        _ => (1280, 34),
    };
    let margin_x = ((script_width as f64) * 0.07).round() as u64;
    let margin_v = 40u64;
    let mut output = format!(
        "[Script Info]\nScriptType: v4.00+\nPlayResX: {script_width}\nPlayResY: {SCRIPT_HEIGHT}\nScaledBorderAndShadow: yes\nWrapStyle: 0\n\n[V4+ Styles]\nFormat: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding\nStyle: Default,{SYSTEM_SANS_FAMILY},{font_size},&H00FFFFFF,&H00FFFFFF,&H22000000,&H78000000,600,0,0,0,100,100,0.4,0,1,1.6,1.2,2,{margin_x},{margin_x},{margin_v},1\n\n[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n"
    );

    let mut previous_start = 0;
    let mut dialogue_count = 0usize;
    for cue in cues {
        let start = cue.start_ms;
        let end = cue.end_ms;
        if end <= start || end > duration_ms || start < previous_start {
            return Err("Subtitle cue timing is invalid".into());
        }
        previous_start = start;
        let text = cue.text.as_str().trim();
        if text.is_empty() {
            return Err("Subtitle cue text is unavailable".into());
        }
        let pieces = split_readable_cues(text);
        let total_chars: usize = pieces.iter().map(|piece| piece.chars().count()).sum();
        let span = end - start;
        let mut cursor = start;
        for (idx, piece) in pieces.iter().enumerate() {
            let piece_end = if idx + 1 == pieces.len() {
                end
            } else {
                let weight = piece.chars().count() as f64 / total_chars.max(1) as f64;
                (cursor + ((span as f64 * weight).round() as u64)).min(end)
            };
            if piece_end <= cursor {
                continue;
            }
            dialogue_count += 1;
            output.push_str(&format!(
                "Dialogue: 0,{},{},Default,,0,0,0,,{{\\blur0.5}}{}\n",
                ass_format::timestamp(cursor),
                ass_format::timestamp(piece_end),
                ass_format::escape_text(piece)
            ));
            cursor = piece_end;
        }
    }
    if dialogue_count == 0 {
        Err("Video transcript is empty".into())
    } else {
        Ok(output)
    }
}

fn is_break_char(ch: char) -> bool {
    matches!(
        ch,
        '。' | '！' | '？' | '；' | '，' | '、' | '.' | '!' | '?' | ';' | ',' | ' '
    )
}

/// Splits long transcript segments into broadcast-readable 1–2 line subtitle chunks.
fn split_readable_cues(raw: &str) -> Vec<String> {
    let normalized: String = raw
        .replace("\r\n", "\n")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    let chars: Vec<char> = normalized.chars().collect();
    if chars.is_empty() {
        return Vec::new();
    }
    let cjk_count = chars.iter().filter(|&&ch| is_cjk(ch)).count();
    let budget = if cjk_count * 3 >= chars.len() {
        CJK_CUE_BUDGET
    } else {
        LATIN_CUE_BUDGET
    };
    if chars.len() <= budget {
        return vec![normalized];
    }
    let mut chunks = Vec::new();
    let mut cursor = 0;
    while cursor < chars.len() {
        if chars.len() - cursor <= budget {
            let tail: String = chars[cursor..].iter().collect();
            let trimmed = tail.trim();
            if !trimmed.is_empty() {
                chunks.push(trimmed.to_owned());
            }
            break;
        }
        let min_cut = cursor + budget / 2;
        let max_cut = (cursor + budget).min(chars.len());
        let mut cut = max_cut;
        for i in (min_cut..=max_cut).rev() {
            if is_break_char(chars[i - 1]) {
                cut = i;
                break;
            }
        }
        let piece: String = chars[cursor..cut].iter().collect();
        let trimmed = piece.trim();
        if !trimmed.is_empty() {
            chunks.push(trimmed.to_owned());
        }
        cursor = cut;
        while cursor < chars.len() && chars[cursor] == ' ' {
            cursor += 1;
        }
    }
    chunks
}

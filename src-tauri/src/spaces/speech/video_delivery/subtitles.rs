use super::super::contracts::SpeechSegment;

pub(super) fn encode(segments: &[SpeechSegment], webvtt: bool) -> Result<String, String> {
    let mut output = if webvtt {
        "WEBVTT\n\n".to_owned()
    } else {
        String::new()
    };
    if segments.is_empty() {
        return Err("The saved dubbing script has no subtitle cues".into());
    }
    for (index, segment) in segments.iter().enumerate() {
        let (Some(start), Some(end)) = (segment.target_start_ms, segment.target_end_ms) else {
            return Err("Dubbing subtitle has no target time window".into());
        };
        if start < 0 || end <= start {
            return Err("Dubbing subtitle time window is invalid".into());
        }
        let text = escape(&segment.text).replace('\r', "");
        let text = if webvtt {
            match segment.speaker_key.as_deref() {
                Some(speaker) => format!("<v {}>{text}</v>", escape(speaker)),
                None => text,
            }
        } else {
            text
        };
        output.push_str(&format!(
            "{}\n{} --> {}\n{}\n\n",
            index + 1,
            timestamp(start, webvtt),
            timestamp(end, webvtt),
            text
        ));
    }
    Ok(output)
}
fn timestamp(ms: i64, webvtt: bool) -> String {
    format!(
        "{:02}:{:02}:{:02}{}{:03}",
        ms / 3_600_000,
        ms / 60_000 % 60,
        ms / 1000 % 60,
        if webvtt { '.' } else { ',' },
        ms % 1000
    )
}

fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
#[path = "tests/subtitles.rs"]
mod tests;

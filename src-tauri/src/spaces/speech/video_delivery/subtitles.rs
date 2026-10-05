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
mod tests {
    use super::*;
    #[test]
    fn exports_exact_timings_and_escaped_webvtt_speaker() {
        let cue = SpeechSegment {
            id: "cue".into(),
            ordinal: 0,
            chapter_key: None,
            speaker_key: Some("Alice & Bob".into()),
            text: "Hello <world>".into(),
            normalized_text: "Hello <world>".into(),
            voice_revision_id: "voice".into(),
            language: "en".into(),
            target_start_ms: Some(1250),
            target_end_ms: Some(3500),
            source_segment_id: None,
            source_segment_revision: None,
            source_text: None,
            state: "pending".into(),
            state_reason: None,
        };
        assert_eq!(encode(&[cue], true).unwrap(), "WEBVTT\n\n1\n00:00:01.250 --> 00:00:03.500\n<v Alice &amp; Bob>Hello &lt;world&gt;</v>\n\n");
    }
}

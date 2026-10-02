use crate::media::transcript::SubtitleCue;

pub(crate) fn from_cues(cues: &[SubtitleCue], duration_ms: u64) -> Result<String, String> {
    let mut srt = String::new();
    let mut previous_start = 0;
    for (index, cue) in cues.iter().enumerate() {
        let start = cue.start_ms;
        let end = cue.end_ms;
        if end <= start || end > duration_ms || start < previous_start {
            return Err("Subtitle cue timing is invalid".into());
        }
        previous_start = start;
        let text = cue.text.as_str();
        if text.trim().is_empty() {
            return Err("Subtitle cue text is unavailable".into());
        }
        srt.push_str(&format!(
            "{}\n{} --> {}\n{}\n\n",
            index + 1,
            timestamp(start),
            timestamp(end),
            escape(text)
        ));
    }
    if srt.is_empty() {
        Err("Video transcript is empty".into())
    } else {
        Ok(srt)
    }
}

fn escape(text: &str) -> String {
    text.replace("\r\n", "\n").replace('\r', "\n")
}

pub(super) fn timestamp(ms: u64) -> String {
    format!(
        "{:02}:{:02}:{:02},{:03}",
        ms / 3_600_000,
        ms / 60_000 % 60,
        ms / 1_000 % 60,
        ms % 1_000
    )
}

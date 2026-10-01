use serde_json::Value;

pub(crate) fn from_cues(value: &Value, duration_ms: u64) -> Result<String, String> {
    let cues = value
        .as_array()
        .ok_or("Precise subtitle timing is unavailable; analyze the video again")?;
    let mut srt = String::new();
    let mut previous_start = 0;
    for (index, cue) in cues.iter().enumerate() {
        let start = time(cue, "startMs")?;
        let end = time(cue, "endMs")?;
        if end <= start || end > duration_ms || start < previous_start {
            return Err("Subtitle cue timing is invalid".into());
        }
        previous_start = start;
        let text = cue
            .get("text")
            .and_then(Value::as_str)
            .filter(|text| !text.trim().is_empty())
            .ok_or("Subtitle cue text is unavailable")?;
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

fn time(value: &Value, key: &str) -> Result<u64, String> {
    value
        .get(key)
        .and_then(Value::as_u64)
        .ok_or_else(|| format!("Subtitle cue {key} is invalid"))
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

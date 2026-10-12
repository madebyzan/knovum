use super::{TextPlacement, TextPreset, VideoEditProject};

pub(super) fn render(project: &VideoEditProject) -> String {
    let (width, height) = (project.output.width, project.output.height);
    let play_res_x = (u64::from(width) * 720 + u64::from(height) / 2) / u64::from(height);
    let center_x = play_res_x / 2;
    let mut output = format!(
        "[Script Info]\nScriptType: v4.00+\nPlayResX: {play_res_x}\nPlayResY: 720\nScaledBorderAndShadow: yes\nWrapStyle: 0\n\n[V4+ Styles]\nFormat: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding\n"
    );
    for (index, item) in project.text_track.items.iter().enumerate() {
        let size = item.style.size;
        let family = &item.style.family;
        let weight = item.style.weight;
        let color = &item.style.color;
        let preset = item.style.preset;
        let (border_style, outline, shadow, back_color) = match preset {
            TextPreset::Plain => (1, 0, 0, "&H00000000"),
            TextPreset::Outline => (1, 2, 0, "&H00000000"),
            TextPreset::Shadow => (1, 0, 2, "&H33000000"),
            TextPreset::Badge => (3, 8, 0, "&H73000000"),
        };
        let alignment = match item.placement {
            TextPlacement::Top => 8,
            TextPlacement::Center => 5,
            TextPlacement::Bottom => 2,
        };
        let primary = ass_color(color);
        output.push_str(&format!(
            "Style: Text{index},{family},{size},{primary},{primary},&H40000000,{back_color},{weight},0,0,0,100,100,0,0,{border_style},{outline},{shadow},{alignment},36,36,58,1\n"
        ));
    }
    output.push_str("\n[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n");
    for (index, item) in project.text_track.items.iter().enumerate() {
        let y = match item.placement {
            TextPlacement::Top => 58,
            TextPlacement::Center => 360,
            TextPlacement::Bottom => 662,
        };
        output.push_str(&format!(
            "Dialogue: 0,{},{},Text{index},,0,0,0,,{{\\pos({center_x},{y})}}{}\n",
            timestamp(item.start_ms),
            timestamp(item.end_ms),
            escape_text(&item.text)
        ));
    }
    output
}

fn ass_color(hex: &str) -> String {
    let rgb = hex.strip_prefix('#').expect("validated text color");
    format!("&H00{}{}{}", &rgb[4..6], &rgb[2..4], &rgb[0..2])
}

fn timestamp(milliseconds: u64) -> String {
    format!(
        "{}:{:02}:{:02}.{:02}",
        milliseconds / 3_600_000,
        milliseconds / 60_000 % 60,
        milliseconds / 1_000 % 60,
        milliseconds % 1_000 / 10
    )
}

fn escape_text(value: &str) -> String {
    let mut escaped = String::new();
    for character in value.replace("\r\n", "\n").chars() {
        match character {
            '\n' | '\r' => escaped.push_str("\\N"),
            '{' => escaped.push_str("\\{"),
            '\\' => escaped.push_str("\\\u{2060}"),
            _ => escaped.push(character),
        }
    }
    escaped
}

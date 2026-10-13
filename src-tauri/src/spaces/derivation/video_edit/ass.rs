use super::contracts::{TextAlign, TextDecoration, TextOverlay};
use super::VideoEditProject;

/// Script resolution is fixed at 720 lines so a text size stays 720p-relative while the exported
/// frame can be any size. Overlays are freely positioned with `\pos` and anchored vertically in the
/// middle, which is exactly what the canvas preview draws.
const SCRIPT_HEIGHT: u64 = 720;

pub(super) fn render(project: &VideoEditProject) -> String {
    let script_width = (u64::from(project.output.width) * SCRIPT_HEIGHT
        + u64::from(project.output.height) / 2)
        / u64::from(project.output.height);
    let mut output = format!(
        "[Script Info]\nScriptType: v4.00+\nPlayResX: {script_width}\nPlayResY: {SCRIPT_HEIGHT}\nScaledBorderAndShadow: yes\nWrapStyle: 2\n\n[V4+ Styles]\nFormat: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding\n"
    );
    for (index, item) in project.text_track.items.iter().enumerate() {
        output.push_str(&style_line(index, item));
    }
    output.push_str("\n[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n");
    for (index, item) in project.text_track.items.iter().enumerate() {
        let x = (item.x * script_width as f64).round() as i64;
        let y = (item.y * SCRIPT_HEIGHT as f64).round() as i64;
        output.push_str(&format!(
            "Dialogue: 0,{start},{end},Text{index},,0,0,0,,{{\\an{anchor}\\pos({x},{y})}}{text}\n",
            start = timestamp(item.start_ms),
            end = timestamp(item.end_ms),
            anchor = anchor(item.style.align),
            text = escape_text(&item.text)
        ));
    }
    output
}

fn style_line(index: usize, item: &TextOverlay) -> String {
    let style = &item.style;
    let primary = ass_color(&style.color);
    let decoration = ass_color(&style.decoration_color);
    let strength = style.decoration_strength;
    let (border_style, outline, shadow) = match style.decoration {
        TextDecoration::Plain => (1, 0, 0),
        TextDecoration::Outline => (1, strength.max(1), 0),
        TextDecoration::Shadow => (1, 0, strength.max(1)),
        TextDecoration::Box => (3, strength.max(1), 0),
    };
    let (outline_color, back_color) = match style.decoration {
        TextDecoration::Shadow => (TRANSPARENT.to_owned(), decoration),
        _ => (decoration, TRANSPARENT.to_owned()),
    };
    format!(
        "Style: Text{index},{family},{size},{primary},{primary},{outline_color},{back_color},{weight},{italic},0,0,100,100,0,0,{border_style},{outline},{shadow},{anchor},0,0,0,1\n",
        family = style.family,
        size = style.size,
        weight = style.weight,
        italic = i32::from(style.italic),
        anchor = anchor(style.align)
    )
}

const TRANSPARENT: &str = "&HFF000000";

fn anchor(align: TextAlign) -> u8 {
    match align {
        TextAlign::Left => 4,
        TextAlign::Center => 5,
        TextAlign::Right => 6,
    }
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

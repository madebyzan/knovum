use super::super::ass_format;
use super::contracts::{TextAlign, TextDecoration, TextOverlay};
use super::VideoEditProject;

/// Script resolution is fixed at 720 lines so a text size stays 720p-relative while the exported
/// frame can be any size. Overlays are freely positioned with `\pos` and anchored vertically in the
/// middle, with smart balanced wrapping (`WrapStyle: 0`) inside 7% safe horizontal margins.
const SCRIPT_HEIGHT: u64 = 720;

pub(super) fn render(project: &VideoEditProject) -> String {
    let script_width = (u64::from(project.output.width) * SCRIPT_HEIGHT
        + u64::from(project.output.height) / 2)
        / u64::from(project.output.height);
    let margin_x = ((script_width as f64) * 0.07).round() as u64;
    let mut output = format!(
        "[Script Info]\nScriptType: v4.00+\nPlayResX: {script_width}\nPlayResY: {SCRIPT_HEIGHT}\nScaledBorderAndShadow: yes\nWrapStyle: 0\n\n[V4+ Styles]\nFormat: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding\n"
    );
    for (index, item) in project.text_track.items.iter().enumerate() {
        output.push_str(&style_line(index, item, margin_x));
    }
    output.push_str("\n[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n");
    for (index, item) in project.text_track.items.iter().enumerate() {
        let x = (item.x * script_width as f64).round() as i64;
        let y = (item.y * SCRIPT_HEIGHT as f64).round() as i64;
        output.push_str(&format!(
            "Dialogue: 0,{start},{end},Text{index},,{margin_x},{margin_x},0,,{{\\an{anchor}\\pos({x},{y})\\blur0.4}}{text}\n",
            start = ass_format::timestamp(item.start_ms),
            end = ass_format::timestamp(item.end_ms),
            anchor = anchor(item.style.align),
            text = ass_format::escape_text(&item.text)
        ));
    }
    output
}

fn style_line(index: usize, item: &TextOverlay, margin_x: u64) -> String {
    let style = &item.style;
    let primary = ass_color(&style.color, "00");
    let strength = style.decoration_strength;
    let (border_style, outline, shadow) = match style.decoration {
        TextDecoration::Plain => (1, 0, 0),
        TextDecoration::Outline => (1, strength.max(1), (strength / 2).max(1)),
        TextDecoration::Shadow => (1, 0, strength.max(1)),
        TextDecoration::Box => (3, strength.max(2), 0),
    };
    let (outline_color, back_color) = match style.decoration {
        TextDecoration::Shadow => (
            TRANSPARENT.to_owned(),
            ass_color(&style.decoration_color, "26"),
        ),
        TextDecoration::Box => (
            ass_color(&style.decoration_color, "38"),
            TRANSPARENT.to_owned(),
        ),
        TextDecoration::Outline => (
            ass_color(&style.decoration_color, "00"),
            SOFT_SHADOW.to_owned(),
        ),
        TextDecoration::Plain => (TRANSPARENT.to_owned(), TRANSPARENT.to_owned()),
    };
    format!(
        "Style: Text{index},{family},{size},{primary},{primary},{outline_color},{back_color},{weight},{italic},0,0,100,100,0,0,{border_style},{outline},{shadow},{anchor},{margin_x},{margin_x},0,1\n",
        family = style.family,
        size = style.size,
        weight = style.weight,
        italic = i32::from(style.italic),
        anchor = anchor(style.align)
    )
}

const TRANSPARENT: &str = "&HFF000000";
const SOFT_SHADOW: &str = "&H88000000";

fn anchor(align: TextAlign) -> u8 {
    match align {
        TextAlign::Left => 4,
        TextAlign::Center => 5,
        TextAlign::Right => 6,
    }
}

fn ass_color(hex: &str, alpha_hex: &str) -> String {
    let rgb = hex.strip_prefix('#').expect("validated text color");
    format!("&H{alpha_hex}{}{}{}", &rgb[4..6], &rgb[2..4], &rgb[0..2])
}

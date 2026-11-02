use super::*;

fn line(text: &str, x: f32, y: f32, confidence: f32) -> RecognizedTextLine {
    RecognizedTextLine {
        text: text.into(),
        confidence,
        x,
        y,
        width: 100.0,
        height: 20.0,
    }
}

#[test]
fn reads_rows_top_to_bottom_and_cells_left_to_right() {
    let lines = [
        line("Total", 10.0, 103.0, 0.9),
        line("¥12,800", 300.0, 100.0, 0.9),
        line("Invoice", 10.0, 50.0, 0.9),
    ];
    assert_eq!(text(&lines), "Invoice\nTotal ¥12,800");
}

#[test]
fn drops_lines_the_recognizer_is_guessing_at() {
    assert_eq!(
        text(&[line("noise", 0.0, 0.0, 0.2), line("kept", 0.0, 50.0, 0.8)]),
        "kept"
    );
}

#[test]
fn nothing_recognized_reads_as_nothing() {
    assert_eq!(text(&[]), "");
}

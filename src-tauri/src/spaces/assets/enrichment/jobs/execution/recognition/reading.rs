use crate::ai::contracts::image_compute::RecognizedTextLine;

/// Below this the recognizer is guessing at marks that are not text.
const MIN_CONFIDENCE: f32 = 0.5;

/// Orders recognized lines into reading text: top to bottom, and left to right within a row. Two
/// lines share a row when their vertical centres sit within half the shorter line's height, which
/// keeps table cells and label/value pairs together on a slightly skewed scan.
pub(super) fn text(lines: &[RecognizedTextLine]) -> String {
    let mut kept: Vec<_> = lines
        .iter()
        .filter(|line| line.confidence >= MIN_CONFIDENCE)
        .collect();
    kept.sort_by(|a, b| centre(a).total_cmp(&centre(b)));
    let mut rows: Vec<Vec<&RecognizedTextLine>> = Vec::new();
    for line in kept {
        match rows.last_mut() {
            Some(row) if shares_row(row[0], line) => row.push(line),
            _ => rows.push(vec![line]),
        }
    }
    rows.iter_mut()
        .map(|row| {
            row.sort_by(|a, b| a.x.total_cmp(&b.x));
            row.iter()
                .map(|line| line.text.as_str())
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn centre(line: &RecognizedTextLine) -> f32 {
    line.y + line.height / 2.0
}

fn shares_row(a: &RecognizedTextLine, b: &RecognizedTextLine) -> bool {
    (centre(a) - centre(b)).abs() < a.height.min(b.height) / 2.0
}

#[cfg(test)]
#[path = "tests/reading.rs"]
mod tests;

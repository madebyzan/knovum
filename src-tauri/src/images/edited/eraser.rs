use super::contract::EraserStroke;

const CELL_SIZE: f64 = 128.0;
const MAX_SEGMENTS: usize = 65_536;
const MAX_CELL_REFERENCES: usize = 4_000_000;

#[derive(Clone, Copy)]
struct Segment {
    stroke: u16,
    start: (f64, f64),
    end: (f64, f64),
    radius: f64,
}

pub(super) struct EraserIndex {
    columns: usize,
    rows: usize,
    segments: Vec<Segment>,
    cells: Vec<Vec<u32>>,
    reference_count: usize,
}

impl EraserIndex {
    pub(super) fn new(width: u32, height: u32, strokes: &[EraserStroke]) -> Result<Self, String> {
        let columns = (f64::from(width) / CELL_SIZE).ceil() as usize;
        let rows = (f64::from(height) / CELL_SIZE).ceil() as usize;
        let mut index = Self {
            columns,
            rows,
            segments: Vec::new(),
            cells: vec![Vec::new(); columns * rows],
            reference_count: 0,
        };
        for (stroke_index, stroke) in strokes.iter().enumerate() {
            index.add_stroke(width, height, stroke_index as u16, stroke)?;
        }
        Ok(index)
    }

    pub(super) fn opacity_at(&self, x: f64, y: f64) -> f64 {
        if self.segments.is_empty() {
            return 1.0;
        }
        let column =
            ((x / CELL_SIZE).floor() as isize).clamp(0, self.columns as isize - 1) as usize;
        let row = ((y / CELL_SIZE).floor() as isize).clamp(0, self.rows as isize - 1) as usize;
        let mut opacity = 1.0;
        let mut active_stroke = None;
        let mut stroke_coverage: f64 = 0.0;
        for segment_index in &self.cells[row * self.columns + column] {
            let segment = self.segments[*segment_index as usize];
            if active_stroke != Some(segment.stroke) {
                opacity *= 1.0 - stroke_coverage;
                active_stroke = Some(segment.stroke);
                stroke_coverage = 0.0;
            }
            stroke_coverage = stroke_coverage.max(coverage((x, y), segment));
        }
        opacity * (1.0 - stroke_coverage)
    }

    fn add_stroke(
        &mut self,
        width: u32,
        height: u32,
        stroke_index: u16,
        stroke: &EraserStroke,
    ) -> Result<(), String> {
        let radius = stroke.diameter * f64::from(width.min(height)) / 2.0;
        let mut points = stroke.points.chunks_exact(2);
        let first = points
            .next()
            .expect("validated eraser stroke has two points");
        let mut start = scaled(first, width, height);
        for point in points {
            if self.segments.len() >= MAX_SEGMENTS {
                return Err("Edited image exceeds the eraser complexity limit".into());
            }
            let end = scaled(point, width, height);
            let segment = Segment {
                stroke: stroke_index,
                start,
                end,
                radius,
            };
            let segment_index = self.segments.len() as u32;
            self.segments.push(segment);
            self.index_segment(segment_index, segment)?;
            start = end;
        }
        Ok(())
    }

    fn index_segment(&mut self, index: u32, segment: Segment) -> Result<(), String> {
        let padding = segment.radius + 0.5;
        let left = cell(segment.start.0.min(segment.end.0) - padding, self.columns);
        let right = cell(segment.start.0.max(segment.end.0) + padding, self.columns);
        let top = cell(segment.start.1.min(segment.end.1) - padding, self.rows);
        let bottom = cell(segment.start.1.max(segment.end.1) + padding, self.rows);
        for row in top..=bottom {
            for column in left..=right {
                if self.reference_count >= MAX_CELL_REFERENCES {
                    return Err("Edited image exceeds the eraser index limit".into());
                }
                self.cells[row * self.columns + column].push(index);
                self.reference_count += 1;
            }
        }
        Ok(())
    }
}

fn scaled(point: &[f64], width: u32, height: u32) -> (f64, f64) {
    (point[0] * f64::from(width), point[1] * f64::from(height))
}

fn cell(coordinate: f64, count: usize) -> usize {
    ((coordinate / CELL_SIZE).floor() as isize).clamp(0, count as isize - 1) as usize
}

fn coverage(point: (f64, f64), segment: Segment) -> f64 {
    let delta = (
        segment.end.0 - segment.start.0,
        segment.end.1 - segment.start.1,
    );
    let length_squared = delta.0 * delta.0 + delta.1 * delta.1;
    let position = if length_squared == 0.0 {
        0.0
    } else {
        (((point.0 - segment.start.0) * delta.0 + (point.1 - segment.start.1) * delta.1)
            / length_squared)
            .clamp(0.0, 1.0)
    };
    let distance = ((point.0 - segment.start.0 - position * delta.0).powi(2)
        + (point.1 - segment.start.1 - position * delta.1).powi(2))
    .sqrt();
    (segment.radius + 0.5 - distance).clamp(0.0, 1.0)
}

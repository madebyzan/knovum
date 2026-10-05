use super::{SignalWarning, SignalWarningKind, SpeechSignalReport};

struct Window {
    start_ms: u64,
    end_ms: u64,
    peak: f64,
    rms: f64,
    invalid: bool,
}
#[derive(Default)]
struct Pending {
    start_ms: Option<u64>,
    peak: Option<f64>,
    rms: Option<f64>,
    nan: Option<f64>,
    infinite: Option<f64>,
    samples: Option<f64>,
}
pub(super) struct WindowScanner {
    windows: Vec<Window>,
    pending: Pending,
    truncated: bool,
}
impl WindowScanner {
    pub(super) fn new() -> Self {
        Self {
            windows: Vec::new(),
            pending: Pending::default(),
            truncated: false,
        }
    }
    pub(super) fn window_count(&self) -> usize {
        self.windows.len()
    }
    pub(super) fn scanned_ms(&self) -> u64 {
        self.windows.last().map_or(0, |window| window.end_ms)
    }
    pub(super) fn read(&mut self, line: &str) -> Result<(), String> {
        if let Some((_, timestamp)) = line.split_once("pts_time:") {
            self.push()?;
            let seconds = timestamp
                .trim()
                .parse::<f64>()
                .map_err(|_| "Invalid signal timestamp")?;
            if !seconds.is_finite() || seconds < 0.0 {
                return Err("Invalid signal timestamp".into());
            }
            self.pending.start_ms = Some((seconds * 1000.0).round() as u64);
        } else if let Some((key, value)) = line.split_once('=') {
            let Some(key) = key.strip_prefix("lavfi.astats.") else {
                return Ok(());
            };
            let value = value
                .parse::<f64>()
                .map_err(|_| "Invalid signal measurement")?;
            match key {
                "Overall.Peak_level" => self.pending.peak = Some(value),
                "Overall.RMS_level" => self.pending.rms = Some(value),
                "Overall.Number_of_samples" => self.pending.samples = Some(value),
                key if key.ends_with(".Number of NaNs") => {
                    self.pending.nan = Some(self.pending.nan.unwrap_or(0.0) + value)
                }
                key if key.ends_with(".Number of Infs") => {
                    self.pending.infinite = Some(self.pending.infinite.unwrap_or(0.0) + value)
                }
                _ => {}
            }
        }
        Ok(())
    }
    fn push(&mut self) -> Result<(), String> {
        let pending = std::mem::take(&mut self.pending);
        let Some(start_ms) = pending.start_ms else {
            return Ok(());
        };
        if start_ms >= 7_200_000 {
            self.truncated = true;
            return Ok(());
        }
        let peak = pending
            .peak
            .ok_or("Signal window has no peak measurement")?;
        let rms = pending.rms.ok_or("Signal window has no RMS measurement")?;
        let nan = pending.nan.ok_or("Signal window has no NaN count")?;
        let infinite = pending
            .infinite
            .ok_or("Signal window has no infinity count")?;
        let samples = pending.samples.ok_or("Signal window has no sample count")?;
        if !samples.is_finite()
            || !(1.0..=4800.0).contains(&samples)
            || !nan.is_finite()
            || !infinite.is_finite()
            || nan < 0.0
            || infinite < 0.0
        {
            return Err("Signal window measurements are invalid".into());
        }
        if self.windows.len() >= 72000
            || self
                .windows
                .last()
                .is_some_and(|previous| start_ms < previous.end_ms)
        {
            return Err("Signal windows exceed their bound or overlap".into());
        }
        self.windows.push(Window {
            start_ms,
            end_ms: (start_ms + (samples / 48.0).round() as u64).min(7_200_000),
            peak,
            rms,
            invalid: nan > 0.0
                || infinite > 0.0
                || peak.is_nan()
                || rms.is_nan()
                || peak == f64::INFINITY
                || rms == f64::INFINITY,
        });
        Ok(())
    }
    pub(super) fn finish(mut self) -> Result<SpeechSignalReport, String> {
        self.push()?;
        let mut active = self
            .windows
            .iter()
            .filter(|window| !window.invalid && window.rms > -50.0)
            .map(|window| window.rms)
            .collect::<Vec<_>>();
        active.sort_by(f64::total_cmp);
        let median = active.get(active.len() / 2).copied();
        let mut warnings = Vec::new();
        for (kind, minimum) in [
            (SignalWarningKind::Invalid, 0),
            (SignalWarningKind::Clipping, 0),
            (SignalWarningKind::Silence, 1000),
            (SignalWarningKind::Quiet, 500),
            (SignalWarningKind::Loud, 500),
        ] {
            let mut start = None;
            let mut end = 0;
            for window in &self.windows {
                let matches = match kind {
                    SignalWarningKind::Invalid => window.invalid,
                    SignalWarningKind::Clipping => !window.invalid && window.peak >= -0.01,
                    SignalWarningKind::Silence => !window.invalid && window.rms <= -50.0,
                    SignalWarningKind::Quiet => {
                        !window.invalid
                            && window.rms > -50.0
                            && median.is_some_and(|median| window.rms < median - 12.0)
                    }
                    SignalWarningKind::Loud => {
                        !window.invalid && median.is_some_and(|median| window.rms > median + 12.0)
                    }
                    SignalWarningKind::Empty => false,
                };
                if matches {
                    start.get_or_insert(window.start_ms);
                    end = window.end_ms;
                } else if let Some(begin) = start.take() {
                    add(&mut warnings, kind, begin, end, minimum);
                }
            }
            if let Some(begin) = start {
                add(&mut warnings, kind, begin, end, minimum);
            }
        }
        if self.windows.is_empty() {
            warnings.push(SignalWarning {
                kind: SignalWarningKind::Empty,
                start_ms: 0,
                end_ms: 0,
            });
        }
        warnings.sort_by_key(|warning| warning.start_ms);
        Ok(SpeechSignalReport {
            scanned_duration_ms: self.scanned_ms(),
            truncated: self.truncated || warnings.len() > 100,
            warnings: warnings.into_iter().take(100).collect(),
        })
    }
}
fn add(
    warnings: &mut Vec<SignalWarning>,
    kind: SignalWarningKind,
    start_ms: u64,
    end_ms: u64,
    minimum: u64,
) {
    if end_ms.saturating_sub(start_ms) >= minimum {
        warnings.push(SignalWarning {
            kind,
            start_ms,
            end_ms,
        });
    }
}

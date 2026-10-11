use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    SampleFormat, Stream, StreamConfig,
};
use tokio::sync::mpsc::UnboundedSender;

use super::engine::DictationAction;
use crate::dictation::contracts::DictationFailure;

const MAX_RECORDING_SECONDS: usize = 120;

#[cfg(test)]
#[path = "tests/recorder.rs"]
mod tests;

pub(crate) struct Recorder {
    stream: Stream,
    samples: Arc<Mutex<Vec<i16>>>,
    sample_rate: u32,
    channels: u16,
}

pub(crate) struct RecordedAudio {
    pub(crate) sample_rate: u32,
    pub(crate) channels: u16,
}

impl Recorder {
    pub(crate) fn start(
        session_id: &str,
        events: UnboundedSender<DictationAction>,
    ) -> Result<Self, DictationFailure> {
        let host = cpal::default_host();
        let device = host.default_input_device().ok_or_else(|| {
            DictationFailure::classified(
                "microphone_unavailable",
                "No default input device is available",
            )
        })?;
        let supported = device
            .default_input_config()
            .map_err(classify_audio_error)?;
        let sample_format = supported.sample_format();
        let config: StreamConfig = supported.into();
        if !(1..=2).contains(&config.channels) {
            return Err(DictationFailure::classified(
                "unsupported_audio_format",
                format!(
                    "Dictation requires one or two channels, got {}",
                    config.channels
                ),
            ));
        }
        let samples = Arc::new(Mutex::new(Vec::new()));
        let maximum =
            config.sample_rate as usize * config.channels as usize * MAX_RECORDING_SECONDS;
        let stream = match sample_format {
            SampleFormat::F32 => build_stream::<f32>(
                &device,
                &config,
                Arc::clone(&samples),
                maximum,
                session_id,
                events,
                f32_to_i16,
            ),
            SampleFormat::I16 => build_stream::<i16>(
                &device,
                &config,
                Arc::clone(&samples),
                maximum,
                session_id,
                events,
                |sample| sample,
            ),
            SampleFormat::U16 => build_stream::<u16>(
                &device,
                &config,
                Arc::clone(&samples),
                maximum,
                session_id,
                events,
                |sample| (i32::from(sample) - 32_768) as i16,
            ),
            value => Err(DictationFailure::classified(
                "unsupported_audio_format",
                format!("Microphone format {value:?} is not supported"),
            )),
        }?;
        stream.play().map_err(classify_audio_error)?;
        Ok(Self {
            stream,
            samples,
            sample_rate: config.sample_rate,
            channels: config.channels,
        })
    }

    pub(crate) fn finish(
        self,
        target: &std::path::Path,
    ) -> Result<RecordedAudio, DictationFailure> {
        drop(self.stream);
        let samples = std::mem::take(&mut *self.samples.lock().map_err(|_| {
            DictationFailure::classified(
                "audio_capture_failed",
                "The captured audio buffer is unavailable",
            )
        })?);
        if samples.is_empty() {
            return Err(DictationFailure::classified(
                "audio_capture_empty",
                "Microphone captured no samples",
            ));
        }
        let spec = hound::WavSpec {
            channels: self.channels,
            sample_rate: self.sample_rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut writer = hound::WavWriter::create(target, spec).map_err(|error| {
            DictationFailure::classified("audio_capture_failed", error.to_string())
        })?;
        for sample in &samples {
            writer.write_sample(*sample).map_err(|error| {
                DictationFailure::classified("audio_capture_failed", error.to_string())
            })?;
        }
        writer.finalize().map_err(|error| {
            DictationFailure::classified("audio_capture_failed", error.to_string())
        })?;
        Ok(RecordedAudio {
            sample_rate: self.sample_rate,
            channels: self.channels,
        })
    }
}

fn build_stream<T: cpal::SizedSample + 'static>(
    device: &cpal::Device,
    config: &StreamConfig,
    samples: Arc<Mutex<Vec<i16>>>,
    maximum: usize,
    session_id: &str,
    events: UnboundedSender<DictationAction>,
    convert: fn(T) -> i16,
) -> Result<Stream, DictationFailure> {
    let signalled = AtomicBool::new(false);
    let failure_session = session_id.to_owned();
    let overflow_session = session_id.to_owned();
    let overflow_events = events.clone();
    device
        .build_input_stream(
            *config,
            move |input: &[T], _| {
                let Ok(mut output) = samples.try_lock() else {
                    return;
                };
                let remaining = maximum.saturating_sub(output.len());
                output.extend(input.iter().copied().take(remaining).map(convert));
                if input.len() > remaining && !signalled.swap(true, Ordering::SeqCst) {
                    let _ = overflow_events.send(DictationAction::DeviceFailed {
                        session_id: overflow_session.clone(),
                        failure: DictationFailure::classified(
                            "recording_limit_reached",
                            "Dictation recording exceeded the two minute limit",
                        ),
                    });
                }
            },
            move |error| {
                if error.kind() == cpal::ErrorKind::DeviceChanged {
                    return;
                }
                let _ = events.send(DictationAction::DeviceFailed {
                    session_id: failure_session.clone(),
                    failure: audio_error(&error),
                });
            },
            None,
        )
        .map_err(classify_audio_error)
}

fn f32_to_i16(sample: f32) -> i16 {
    (sample.clamp(-1.0, 1.0) * f32::from(i16::MAX)).round() as i16
}

fn classify_audio_error(error: cpal::Error) -> DictationFailure {
    audio_error(&error)
}

fn audio_error(error: &cpal::Error) -> DictationFailure {
    let code = if error.kind() == cpal::ErrorKind::PermissionDenied {
        "microphone_permission_denied"
    } else {
        "microphone_unavailable"
    };
    DictationFailure::classified(code, error.to_string())
}

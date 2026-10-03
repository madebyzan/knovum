pub(crate) mod acquisition;
pub(crate) mod audio_preparation;
pub(crate) mod compute_budget;
pub(crate) mod prepared_audio;
pub(crate) mod subprocess;
pub(crate) mod subtitle_timing;
pub(crate) mod transcript;
pub(crate) mod waveform;
pub(crate) mod workspace;

pub(crate) const TRANSCRIPTION_CANCELLED: &str = "media-transcription-cancelled";

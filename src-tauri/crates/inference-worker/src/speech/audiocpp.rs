//! Ownership of the pinned audio.cpp C ABI.
//!
//! Every audio.cpp-backed capability opens one [`Session`] and drives it with [`Request`]s. Handle
//! lifetimes, status checks and the borrowed views a result hands out are kept here once, so no
//! capability repeats the FFI discipline or can free a handle twice.

use super::ffi::{self, audio::*};
use crate::initialization::VerifiedRuntime;
use std::{
    ffi::{c_char, CStr},
    path::Path,
    ptr,
};

/// The C ABI this worker is built against (`audiocpp.h` 0.2.0, packed major/minor/patch).
const ABI_VERSION: u32 = 0x00_02_00;

/// What a capability opens: one model file, one family and one task on one backend.
pub(super) struct SessionSpec<'a> {
    pub(super) model: &'a Path,
    pub(super) family: &'a CStr,
    pub(super) task: &'a CStr,
    /// The host-probed device: `cpu`, `mps` or `vulkan`.
    pub(super) device: &'a str,
    pub(super) threads: u8,
    /// The error code every failure of this session reports under.
    pub(super) failure: &'static str,
}

pub(super) struct Session {
    api: AudioApi,
    handle: *mut audiocpp_session,
    failure: &'static str,
}

impl Drop for Session {
    fn drop(&mut self) {
        unsafe { self.api.audiocpp_session_free(self.handle) }
    }
}

impl Session {
    pub(super) fn open(runtime: &VerifiedRuntime, spec: SessionSpec<'_>) -> Result<Self, String> {
        let api =
            unsafe { AudioApi::from_library(ffi::load(&ffi::library(runtime, "audiocpp")?)?) }
                .map_err(|error| format!("runtime_mismatch: {error}"))?;
        if unsafe { api.audiocpp_abi_version() } != ABI_VERSION {
            return Err("runtime_mismatch: audio.cpp ABI differs".into());
        }
        let backend = match spec.device {
            "cpu" => c"cpu",
            "mps" => c"metal",
            "vulkan" => c"vulkan",
            _ => return Err("unsupported_device: audio.cpp backend is invalid".into()),
        };
        let path = ffi::string(
            spec.model
                .to_str()
                .ok_or("model_corrupt: model path is invalid")?,
        )?;
        let mut registry = ptr::null_mut();
        let mut model = ptr::null_mut();
        let mut handle = ptr::null_mut();
        check(&api, spec.failure, unsafe {
            api.audiocpp_registry_create(ptr::null(), &mut registry)
        })?;
        let opened = (|| {
            let config = audiocpp_model_config {
                family_hint: spec.family.as_ptr(),
                config_id: ptr::null(),
                weight_id: ptr::null(),
                model_spec_override: ptr::null(),
            };
            check(&api, spec.failure, unsafe {
                api.audiocpp_model_load(registry, path.as_ptr(), &config, ptr::null(), &mut model)
            })?;
            let config = audiocpp_backend_config {
                backend: backend.as_ptr(),
                device: 0,
                threads: i32::from(spec.threads),
            };
            check(&api, spec.failure, unsafe {
                api.audiocpp_session_create(
                    model,
                    spec.task.as_ptr(),
                    c"offline".as_ptr(),
                    &config,
                    ptr::null(),
                    &mut handle,
                )
            })
        })();
        // A session keeps its model and registry alive itself.
        unsafe {
            api.audiocpp_model_free(model);
            api.audiocpp_registry_free(registry);
        }
        if let Err(error) = opened {
            unsafe { api.audiocpp_session_free(handle) };
            return Err(error);
        }
        if handle.is_null() {
            return Err("model_corrupt: audio.cpp session is absent".into());
        }
        Ok(Self {
            api,
            handle,
            failure: spec.failure,
        })
    }

    pub(super) fn request(&self) -> Result<Request<'_>, String> {
        let handle = unsafe { self.api.audiocpp_request_create() };
        if handle.is_null() {
            return Err("resource_exhausted: audio.cpp request allocation failed".into());
        }
        Ok(Request {
            session: self,
            handle,
        })
    }

    pub(super) fn run(&self, request: &Request<'_>) -> Result<Output<'_>, String> {
        let mut handle = ptr::null_mut();
        let status = unsafe {
            self.api
                .audiocpp_session_run(self.handle, request.handle, &mut handle)
        };
        // Owned before the status is read, so a result handed out with a failure is still freed.
        let output = Output {
            session: self,
            handle,
        };
        self.check(status)?;
        if handle.is_null() {
            return Err(format!("{}: audio.cpp returned no result", self.failure));
        }
        Ok(output)
    }

    fn check(&self, status: audiocpp_status) -> Result<(), String> {
        check(&self.api, self.failure, status)
    }
}

pub(super) struct Request<'a> {
    session: &'a Session,
    handle: *mut audiocpp_request,
}

impl Drop for Request<'_> {
    fn drop(&mut self) {
        unsafe { self.session.api.audiocpp_request_free(self.handle) }
    }
}

impl Request<'_> {
    /// The text to speak or align, without touching the request's `language` option.
    pub(super) fn text(&mut self, text: &str) -> Result<(), String> {
        let text = ffi::string(text)?;
        self.session.check(unsafe {
            self.session
                .api
                .audiocpp_request_set_text(self.handle, text.as_ptr(), ptr::null())
        })
    }

    pub(super) fn text_language(&mut self, language: &str) -> Result<(), String> {
        let language = ffi::string(language)?;
        self.session.check(unsafe {
            self.session
                .api
                .audiocpp_request_set_text_language(self.handle, language.as_ptr())
        })
    }

    /// The audio the task reads, interleaved.
    pub(super) fn audio(&mut self, audio: AudioView<'_>) -> Result<(), String> {
        self.session.check(unsafe {
            self.session.api.audiocpp_request_set_audio(
                self.handle,
                audio.samples.as_ptr(),
                audio.frames(),
                audio.sample_rate as i32,
                i32::from(audio.channels),
            )
        })
    }

    /// The speaker a cloning family reproduces, interleaved.
    pub(super) fn voice_audio(&mut self, audio: AudioView<'_>) -> Result<(), String> {
        self.session.check(unsafe {
            self.session.api.audiocpp_request_set_voice_audio(
                self.handle,
                audio.samples.as_ptr(),
                audio.frames(),
                audio.sample_rate as i32,
                i32::from(audio.channels),
            )
        })
    }

    pub(super) fn option(&mut self, key: &CStr, value: &str) -> Result<(), String> {
        let value = ffi::string(value)?;
        self.session.check(unsafe {
            self.session
                .api
                .audiocpp_request_set_option(self.handle, key.as_ptr(), value.as_ptr())
        })
    }
}

/// Interleaved PCM, borrowed from its owner.
#[derive(Clone, Copy)]
pub(super) struct AudioView<'a> {
    pub(super) samples: &'a [f32],
    pub(super) sample_rate: u32,
    pub(super) channels: u16,
}

impl AudioView<'_> {
    pub(super) fn frames(&self) -> usize {
        self.samples.len() / usize::from(self.channels)
    }
}

/// One aligned word, in samples of the request's audio.
pub(super) struct Word {
    pub(super) text: String,
    pub(super) start: u64,
    pub(super) end: u64,
}

pub(super) struct Output<'a> {
    session: &'a Session,
    handle: *mut audiocpp_result,
}

impl Drop for Output<'_> {
    fn drop(&mut self) {
        unsafe { self.session.api.audiocpp_result_free(self.handle) }
    }
}

impl Output<'_> {
    /// The transcript a recognition task produced, beside its detected or requested language
    /// label. An empty language means the model found no speech to attribute a language to.
    pub(super) fn transcript(&self) -> Result<(String, String), String> {
        let mut text = ptr::null();
        let mut language = ptr::null();
        self.session.check(unsafe {
            self.session
                .api
                .audiocpp_result_text(self.handle, &mut text, &mut language)
        })?;
        Ok((self.text(text)?, self.text(language)?))
    }

    pub(super) fn words(&self) -> Result<Vec<Word>, String> {
        let api = &self.session.api;
        let count = unsafe { api.audiocpp_result_word_count(self.handle) };
        let mut words = Vec::with_capacity(count);
        for index in 0..count {
            let mut text = ptr::null();
            let (mut start, mut end) = (0_i64, 0_i64);
            self.session.check(unsafe {
                api.audiocpp_result_word(
                    self.handle,
                    index,
                    &mut text,
                    &mut start,
                    &mut end,
                    ptr::null_mut(),
                )
            })?;
            if start < 0 || end <= start {
                return Err(self.invalid("a word timeline is invalid"));
            }
            words.push(Word {
                text: self.text(text)?,
                start: start as u64,
                end: end as u64,
            });
        }
        Ok(words)
    }

    /// The single audio stream a synthesis produced.
    pub(super) fn audio(&self) -> Result<AudioView<'_>, String> {
        let mut samples = ptr::null();
        let (mut frames, mut rate, mut channels) = (0_usize, 0_i32, 0_i32);
        self.session.check(unsafe {
            self.session.api.audiocpp_result_audio(
                self.handle,
                &mut samples,
                &mut frames,
                &mut rate,
                &mut channels,
            )
        })?;
        self.view(samples, frames, rate, channels)
    }

    /// Every named stream a multi-output task produced, such as separated stems.
    pub(super) fn named_audio(&self) -> Result<Vec<(String, AudioView<'_>)>, String> {
        let api = &self.session.api;
        let count = unsafe { api.audiocpp_result_named_audio_count(self.handle) };
        let mut streams = Vec::with_capacity(count);
        for index in 0..count {
            let (mut id, mut samples) = (ptr::null(), ptr::null());
            let (mut frames, mut rate, mut channels) = (0_usize, 0_i32, 0_i32);
            self.session.check(unsafe {
                api.audiocpp_result_named_audio(
                    self.handle,
                    index,
                    &mut id,
                    &mut samples,
                    &mut frames,
                    &mut rate,
                    &mut channels,
                )
            })?;
            streams.push((self.text(id)?, self.view(samples, frames, rate, channels)?));
        }
        Ok(streams)
    }

    fn view(
        &self,
        samples: *const f32,
        frames: usize,
        rate: i32,
        channels: i32,
    ) -> Result<AudioView<'_>, String> {
        let (Ok(sample_rate), Ok(channels)) = (u32::try_from(rate), u16::try_from(channels)) else {
            return Err(self.invalid("an audio format is invalid"));
        };
        let length = frames.checked_mul(usize::from(channels));
        let (Some(length), false) = (length, samples.is_null()) else {
            return Err(self.invalid("an audio stream is absent"));
        };
        if sample_rate == 0 || channels == 0 || frames == 0 {
            return Err(self.invalid("an audio stream is empty"));
        }
        // Borrowed from the result, which this view cannot outlive.
        let samples = unsafe { std::slice::from_raw_parts(samples, length) };
        if samples.iter().any(|sample| !sample.is_finite()) {
            return Err(self.invalid("an audio stream is not finite"));
        }
        Ok(AudioView {
            samples,
            sample_rate,
            channels,
        })
    }

    fn text(&self, text: *const c_char) -> Result<String, String> {
        if text.is_null() {
            return Err(self.invalid("a text is absent"));
        }
        unsafe { CStr::from_ptr(text) }
            .to_str()
            .map(str::to_owned)
            .map_err(|_| self.invalid("a text is not UTF-8"))
    }

    fn invalid(&self, detail: &str) -> String {
        format!("invalid_output: audio.cpp result {detail}")
    }
}

fn check(api: &AudioApi, failure: &str, status: audiocpp_status) -> Result<(), String> {
    if status == 0 {
        return Ok(());
    }
    let error = unsafe { api.audiocpp_last_error() };
    let message = if error.is_null() {
        "audio.cpp call failed".into()
    } else {
        unsafe { CStr::from_ptr(error) }.to_string_lossy()
    };
    Err(format!("{failure}: {message}"))
}

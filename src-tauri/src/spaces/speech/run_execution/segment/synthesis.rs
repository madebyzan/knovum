use crate::{
    ai::{
        contracts::speech_wire::MIN_DURATION_SCALE,
        local_runtime::{admission::Priority, speech},
    },
    spaces::{
        runs::AttemptClaim,
        speech::{
            contracts::{SpeechSegmentExecution, VoiceExecution},
            execution::{
                parse_config, synthesis_params, DurationFit, PreparedSynthesis, SynthesisAttempt,
                Take,
            },
        },
    },
};

/// Synthesizes one line. A line without a physical video window renders once at the voice
/// revision's frozen seed. A line with one is brought to its window the way its engine allows
/// ([`DurationFit`]): an engine that controls duration regenerates an overrunning line at the
/// scale that fits; any other spends its takes over derived seeds, returns the first that fits,
/// and otherwise keeps the shortest overrun so the time-fitting stage has the least to compress.
pub(super) async fn synthesize(
    app: &tauri::AppHandle,
    claim: &AttemptClaim,
    voice: &VoiceExecution,
    segment: &SpeechSegmentExecution,
    fit: Option<(i64, i64)>,
    attempt: SynthesisAttempt<'_>,
) -> Result<speech::SynthesizedAudio, String> {
    let model = crate::spaces::speech::execution::prepare_synthesis(
        app,
        &voice.engine_id,
        &voice.model_spec_id,
        &claim.attempt_id,
        "Preparing speech synthesis",
        &claim.cancellation,
        &mut |_| {},
    )
    .await?;
    let line = Line {
        app,
        claim,
        voice,
        segment,
        model: &model,
        attempt,
    };
    let seed = parse_config(&voice.synthesis_config)?.seed;
    let natural = Take {
        seed,
        duration_scale: None,
    };
    let Some(window_ms) = fit.map(|(start_ms, end_ms)| (end_ms - start_ms) as u64) else {
        return line.render(natural, attempt.input_revision).await;
    };
    match DurationFit::of(&voice.engine_id)? {
        DurationFit::Scale => {
            let audio = line.render(natural, attempt.input_revision).await?;
            let spoken_ms = duration_ms(&audio);
            if spoken_ms <= window_ms {
                return Ok(audio);
            }
            let scale = (window_ms as f32 / spoken_ms as f32).clamp(MIN_DURATION_SCALE, 1.0);
            line.render(
                Take {
                    seed,
                    duration_scale: Some(scale),
                },
                &format!("{}-scaled", attempt.input_revision),
            )
            .await
        }
        DurationFit::Takes(takes) => {
            let mut best: Option<speech::SynthesizedAudio> = None;
            for take in 0..takes {
                let input_revision = if take == 0 {
                    attempt.input_revision.to_owned()
                } else {
                    format!("{}-take{take}", attempt.input_revision)
                };
                let audio = line
                    .render(
                        Take {
                            seed: seed.wrapping_add(take),
                            duration_scale: None,
                        },
                        &input_revision,
                    )
                    .await?;
                let spoken_ms = duration_ms(&audio);
                if spoken_ms <= window_ms {
                    return Ok(audio);
                }
                if best
                    .as_ref()
                    .is_none_or(|previous| spoken_ms < duration_ms(previous))
                {
                    best = Some(audio);
                }
            }
            best.ok_or_else(|| "invalid_state: duration fitting produced no synthesis take".into())
        }
    }
}

/// Everything one rendering of a line shares with the others.
struct Line<'a> {
    app: &'a tauri::AppHandle,
    claim: &'a AttemptClaim,
    voice: &'a VoiceExecution,
    segment: &'a SpeechSegmentExecution,
    model: &'a PreparedSynthesis,
    attempt: SynthesisAttempt<'a>,
}

impl Line<'_> {
    async fn render(
        &self,
        take: Take,
        input_revision: &str,
    ) -> Result<speech::SynthesizedAudio, String> {
        if self.claim.cancellation.is_cancelled() {
            return Err("cancelled: speech synthesis was cancelled".into());
        }
        let params = synthesis_params(
            self.app,
            self.voice,
            &self.segment.normalized_text,
            self.segment.delivery_instruction.as_deref(),
            take,
            SynthesisAttempt {
                input_revision,
                ..self.attempt
            },
        )?;
        self.model
            .synthesize(
                self.app,
                params,
                speech::SynthesisExecution {
                    cancellation: &self.claim.cancellation,
                    priority: Priority::Background,
                    on_chunk: None,
                },
            )
            .await
    }
}

fn duration_ms(audio: &speech::SynthesizedAudio) -> u64 {
    audio.sample_count * 1_000 / u64::from(audio.sample_rate).max(1)
}

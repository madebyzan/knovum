use crate::{
    ai::local_runtime::{admission::Priority, speech},
    spaces::{
        runs::AttemptClaim,
        speech::{
            contracts::{SpeechSegmentExecution, VoiceExecution},
            execution::{parse_config, synthesis_params, SynthesisAttempt, DURATION_FIT_TAKES},
        },
    },
};

/// Synthesizes one line. A line without a physical video window renders once at the voice
/// revision's frozen seed. A duration-fitted line spends up to [`DURATION_FIT_TAKES`] takes over
/// procedurally derived seeds, returns the first take that fits, and otherwise keeps the shortest
/// overrun so the time-fitting stage has the least material to compress.
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
    let base_seed = parse_config(&voice.synthesis_config)?.seed;
    let window_ms = fit.map(|(start_ms, end_ms)| (end_ms - start_ms) as u64);
    let takes = if window_ms.is_some() {
        DURATION_FIT_TAKES
    } else {
        1
    };
    let mut best: Option<speech::SynthesizedAudio> = None;
    for take in 0..takes {
        if claim.cancellation.is_cancelled() {
            return Err("cancelled: speech synthesis was cancelled".into());
        }
        let input_revision = if take == 0 {
            attempt.input_revision.to_owned()
        } else {
            format!("{}-take{take}", attempt.input_revision)
        };
        let params = synthesis_params(
            app,
            voice,
            &segment.normalized_text,
            segment.delivery_instruction.as_deref(),
            base_seed.wrapping_add(take),
            SynthesisAttempt {
                input_revision: &input_revision,
                ..attempt
            },
        )?;
        let audio = model
            .synthesize(
                app,
                params,
                speech::SynthesisExecution {
                    cancellation: &claim.cancellation,
                    priority: Priority::Background,
                    on_chunk: None,
                },
            )
            .await?;
        let Some(window_ms) = window_ms else {
            return Ok(audio);
        };
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
    best.ok_or_else(|| "invalid_state: duration fitting produced no synthesis take".to_owned())
}

fn duration_ms(audio: &speech::SynthesizedAudio) -> u64 {
    audio.sample_count * 1_000 / u64::from(audio.sample_rate).max(1)
}

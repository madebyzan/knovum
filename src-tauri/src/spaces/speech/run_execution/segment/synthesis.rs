use crate::{
    ai::local_runtime::{admission::Priority, speech},
    spaces::runs::AttemptClaim,
};

pub(super) struct SegmentTiming {
    pub(super) project_type: String,
    pub(super) start_ms: Option<i64>,
    pub(super) end_ms: Option<i64>,
}

pub(super) fn validate_segment_timing(
    timing: &SegmentTiming,
    sample_rate: u32,
    sample_count: u64,
) -> Result<(), String> {
    if timing.project_type != "video_dubbing" {
        return Ok(());
    }
    let (Some(start), Some(end)) = (timing.start_ms, timing.end_ms) else {
        return Err("Video dubbing segment has no target time window".into());
    };
    let window_ms = u128::try_from(end - start)
        .map_err(|_| "Video dubbing segment has an invalid target time window")?;
    let actual = u128::from(sample_count)
        .checked_mul(1_000)
        .and_then(|value| value.checked_mul(10))
        .ok_or("Synthesized speech duration overflow")?;
    let target = window_ms
        .checked_mul(u128::from(sample_rate))
        .ok_or("Video target duration overflow")?;
    if actual < target.saturating_mul(9) || actual > target.saturating_mul(11) {
        return Err(format!(
            "Synthesized speech duration does not fit target window {start}-{end} ms within the allowed 0.9-1.1 ratio"
        ));
    }
    Ok(())
}

pub(super) async fn synthesize(
    app: &tauri::AppHandle,
    claim: &AttemptClaim,
    voice: &crate::spaces::speech::contracts::VoiceExecution,
    segment: &crate::spaces::speech::contracts::SpeechSegmentExecution,
    attempt: crate::spaces::speech::execution::SynthesisAttempt<'_>,
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
    let params = crate::spaces::speech::execution::synthesis_params(
        app,
        voice,
        &segment.normalized_text,
        segment.delivery_instruction.as_deref(),
        attempt,
    )?;
    model
        .synthesize(
            app,
            params,
            speech::SynthesisExecution {
                cancellation: &claim.cancellation,
                priority: Priority::Background,
                on_chunk: None,
            },
        )
        .await
}

use crate::{
    ai::local_runtime::{admission::Priority, speech},
    spaces::runs::AttemptClaim,
};

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

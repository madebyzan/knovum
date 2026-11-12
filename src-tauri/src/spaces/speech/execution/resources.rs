use crate::{
    ai::{
        contracts::{runtime::RuntimeDeviceProfile, speech_wire::SpeechSynthesizeParams},
        local_runtime::{device_profiles, speech},
    },
    local_resources::preparation::{
        self, contracts::ResourcePreparationProgress, PreparedModels, PreparedVoiceModel,
    },
};
use tauri::AppHandle;
use tokio_util::sync::CancellationToken;

/// The model a voice revision speaks with, prepared for one job: MOSS-TTS-Nano on the CPU, or an
/// audio.cpp voice model (VoxCPM2 or a cloning family) on its probed backend.
pub(in crate::spaces::speech) enum PreparedSynthesis {
    MossNano(PreparedModels),
    Voice {
        model: PreparedVoiceModel,
        profile: RuntimeDeviceProfile,
    },
}

pub(in crate::spaces::speech) async fn prepare_synthesis(
    app: &AppHandle,
    engine: &str,
    model_spec_id: &str,
    job_id: &str,
    title: &str,
    cancellation: &CancellationToken,
    progress: &mut impl FnMut(ResourcePreparationProgress),
) -> Result<PreparedSynthesis, String> {
    if engine == "moss_nano" {
        if model_spec_id != super::MOSS_NANO_MODEL_SPEC_ID {
            return Err("runtime_mismatch: MOSS engine and model specification differ".into());
        }
        return Ok(PreparedSynthesis::MossNano(
            preparation::prepare_models_for_job(
                app,
                preparation::contracts::ResourcePreparationTarget::SpeechSynthesis,
                job_id,
                title,
                cancellation,
                progress,
            )
            .await?,
        ));
    }
    let model = preparation::prepare_voice_for_job(
        app,
        model_spec_id,
        job_id,
        title,
        cancellation,
        progress,
    )
    .await?;
    if model.model().voice_engine().map(|engine| engine.id()) != Some(engine) {
        return Err("runtime_mismatch: voice engine and model specification differ".into());
    }
    let profile = synthesis_device_profile(app, engine, model_spec_id, cancellation).await?;
    Ok(PreparedSynthesis::Voice { model, profile })
}

impl PreparedSynthesis {
    pub(in crate::spaces::speech) async fn synthesize(
        &self,
        app: &AppHandle,
        params: SpeechSynthesizeParams,
        execution: speech::SynthesisExecution<'_>,
    ) -> Result<speech::SynthesizedAudio, String> {
        match self {
            Self::MossNano(model) => {
                speech::synthesize(
                    app,
                    model,
                    super::moss_nano_device_profile(),
                    params,
                    execution,
                )
                .await
            }
            Self::Voice { model, profile } => {
                speech::synthesize_voice(app, model, profile.clone(), params, execution).await
            }
        }
    }
}

pub(in crate::spaces::speech) async fn synthesis_device_profile(
    app: &AppHandle,
    engine: &str,
    model_spec_id: &str,
    cancellation: &CancellationToken,
) -> Result<RuntimeDeviceProfile, String> {
    match engine {
        "moss_nano" if model_spec_id == super::MOSS_NANO_MODEL_SPEC_ID => {
            Ok(super::moss_nano_device_profile())
        }
        "moss_nano" => Err("runtime_mismatch: MOSS engine and model specification differ".into()),
        _ => {
            let runtime = crate::local_resources::runtimes::inference::resolve(app)?;
            device_profiles::voice::profile(&runtime, model_spec_id, cancellation).await
        }
    }
}

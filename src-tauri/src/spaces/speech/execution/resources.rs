use crate::{
    ai::{
        contracts::{runtime::RuntimeDeviceProfile, speech_wire::SpeechSynthesizeParams},
        local_runtime::{device_profiles, speech},
    },
    local_resources::preparation::{
        self, contracts::ResourcePreparationProgress, PreparedModels, PreparedVoxCpm2Model,
    },
};
use tauri::AppHandle;
use tokio_util::sync::CancellationToken;

pub(in crate::spaces::speech) enum PreparedSynthesis {
    MossNano(PreparedModels),
    VoxCpm2 {
        model: PreparedVoxCpm2Model,
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
    match engine {
        "moss_nano" => {
            if model_spec_id != super::MOSS_NANO_MODEL_SPEC_ID {
                return Err("runtime_mismatch: MOSS engine and model specification differ".into());
            }
            Ok(PreparedSynthesis::MossNano(
                preparation::prepare_models_for_job(
                    app,
                    preparation::contracts::ResourcePreparationTarget::SpeechSynthesis,
                    job_id,
                    title,
                    cancellation,
                    progress,
                )
                .await?,
            ))
        }
        "voxcpm2" => {
            device_profiles::voice::require_memory(model_spec_id)?;
            let model = preparation::prepare_voxcpm2_for_job(
                app,
                model_spec_id,
                job_id,
                title,
                cancellation,
                progress,
            )
            .await?;
            let profile =
                synthesis_device_profile(app, engine, model_spec_id, cancellation).await?;
            Ok(PreparedSynthesis::VoxCpm2 { model, profile })
        }
        _ => Err("Voice synthesis engine is unsupported".into()),
    }
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
            Self::VoxCpm2 { model, profile } => {
                speech::synthesize_voxcpm2(app, model, profile.clone(), params, execution).await
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
        "voxcpm2" => {
            let runtime = crate::local_resources::runtimes::inference::resolve(app)?;
            device_profiles::voice::profile(&runtime, model_spec_id, cancellation).await
        }
        _ => Err("Speech voice uses an unsupported engine".into()),
    }
}

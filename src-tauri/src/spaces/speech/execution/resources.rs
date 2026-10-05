use crate::{
    ai::{
        contracts::{runtime::RuntimeDeviceProfile, speech_wire::SpeechSynthesizeParams},
        local_runtime::{device_profiles, speech},
    },
    local_resources::preparation::{
        self, contracts::ResourcePreparationProgress, PreparedKokoroModel, PreparedVoxCpm2Model,
    },
};
use tauri::AppHandle;
use tokio_util::sync::CancellationToken;

pub(in crate::spaces::speech) enum PreparedSynthesis {
    Kokoro(PreparedKokoroModel),
    VoxCpm2 {
        model: PreparedVoxCpm2Model,
        profile: RuntimeDeviceProfile,
    },
}

pub(in crate::spaces::speech) async fn prepare_synthesis(
    app: &AppHandle,
    engine: &str,
    job_id: &str,
    title: &str,
    cancellation: &CancellationToken,
    progress: &mut impl FnMut(ResourcePreparationProgress),
) -> Result<PreparedSynthesis, String> {
    match engine {
        "kokoro" => Ok(PreparedSynthesis::Kokoro(
            preparation::prepare_kokoro_for_job(app, job_id, title, cancellation, progress).await?,
        )),
        "voxcpm2" => {
            let profile = device_profiles::voxcpm2()?;
            let model =
                preparation::prepare_voxcpm2_for_job(app, job_id, title, cancellation, progress)
                    .await?;
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
            Self::Kokoro(model) => {
                speech::synthesize(
                    app,
                    model,
                    super::kokoro_device_profile(),
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

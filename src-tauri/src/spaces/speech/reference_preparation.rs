use crate::spaces::speech::asset_publication::PublicationOwnerKind;

use std::{fs, path::Path};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};

use super::{asset_publication, cancellation, voice_assets, voice_reference_validation};
use crate::{
    foundation::app_error::AppError,
    spaces::{assets, db},
};

#[derive(Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct PrepareVoiceReferenceRequest {
    pub job_id: String,
    pub source_path: String,
    pub language: String,
    pub start_ms: u64,
    pub end_ms: u64,
}

#[derive(Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct PreparedVoiceReference {
    pub source_path: String,
    pub source_revision: String,
    pub start_ms: u64,
    pub end_ms: u64,
    pub transcript: String,
}

#[tauri::command]
pub async fn prepare_voice_reference(
    app: AppHandle,
    request: PrepareVoiceReferenceRequest,
) -> Result<PreparedVoiceReference, AppError> {
    let job_id = uuid::Uuid::parse_str(&request.job_id)
        .map_err(|_| "Voice reference preparation identity must be a UUID")?;
    if job_id.to_string() != request.job_id {
        return Err("Voice reference preparation identity must use canonical UUID form".into());
    }
    if request.end_ms <= request.start_ms
        || !(5000..=30000).contains(&(request.end_ms - request.start_ms))
        || request.end_ms > 43_200_000
    {
        return Err(
            "Voice reference selection must be five to thirty seconds within a twelve-hour source"
                .into(),
        );
    }
    let (cancellation, _lease) = cancellation::acquire(&request.job_id)?;
    let source = Path::new(&request.source_path).to_owned();
    assets::authorize_source(&app, &source)?;
    let length = fs::metadata(&source)
        .map_err(|error| error.to_string())?
        .len();
    if length == 0 {
        return Err("Voice reference source is empty".into());
    }
    let pool = db::pool(&app).await?;
    let directory = assets::asset_directory(&app)?
        .join("speech")
        .join("reference-preparation");
    fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
    let input = directory.join(format!("{}.input", request.job_id));
    let output = directory.join(format!("{}.wav", request.job_id));
    if input.exists() || output.exists() {
        return Err("Reference preparation identity is already occupied".into());
    }
    let publication_id = asset_publication::stage_unowned(
        pool,
        &crate::spaces::authorization::RunScope::Application,
        asset_publication::PendingAsset {
            owner_kind: PublicationOwnerKind::VoiceReference,
            owner_id: &request.job_id,
            asset_path: &output,
            part_path: Some(&input),
            sha256: &"0".repeat(64),
        },
    )
    .await?;
    let preparation: Result<PreparedVoiceReference, String> = async {
        let original = source.to_owned();
        let snapshot = input.clone();
        let reference = output.clone();
        let cancel = cancellation.clone();
        let handle = app.clone();
        let interval = (request.start_ms, request.end_ms);
        let (revision, audio) = crate::foundation::blocking::run(move || -> Result<_, String> {
            let (revision, mut audio) = voice_assets::prepare_reference_clip(
                &handle, &original, &snapshot, length, &cancel, interval,
            )?;
            fs::rename(&snapshot, &reference).map_err(|error| error.to_string())?;
            audio.path = reference.to_string_lossy().into_owned();
            Ok((revision, audio))
        })
        .await?;
        let transcript = voice_reference_validation::validate(
            &app,
            &request.job_id,
            &request.language,
            audio,
            None,
            &cancellation,
        )
        .await?;
        if cancellation.is_cancelled() {
            return Err("cancelled".into());
        }
        Ok(PreparedVoiceReference {
            source_path: request.source_path,
            source_revision: revision,
            start_ms: request.start_ms,
            end_ms: request.end_ms,
            transcript,
        })
    }
    .await;
    if let Err(cleanup) = asset_publication::abandon(&app, pool, &publication_id).await {
        return Err(format!(
            "{}; reference preparation cleanup failed: {cleanup}",
            preparation
                .err()
                .unwrap_or_else(|| "Reference prepared".into())
        )
        .into());
    }
    preparation.map_err(Into::into)
}

#[tauri::command]
pub fn authorize_voice_reference_source(
    app: AppHandle,
    source_path: String,
) -> Result<String, AppError> {
    let source = Path::new(&source_path);
    assets::authorize_source(&app, source)?;
    let metadata = fs::metadata(source).map_err(|error| error.to_string())?;
    if !metadata.is_file() || metadata.len() == 0 {
        return Err("Voice reference must be a nonempty regular media file".into());
    }
    app.asset_protocol_scope()
        .allow_file(source)
        .map_err(|error| error.to_string())?;
    Ok(source_path)
}

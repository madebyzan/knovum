use std::time::{SystemTime, UNIX_EPOCH};
use tauri::Runtime;
use tokio_util::sync::CancellationToken;

use crate::{
    ai::local_runtime::compute_budget::ComputeBudget,
    ai::{
        contracts::{
            runtime::RuntimeDeviceProfile,
            speech_wire::{
                AsrProfile, AudioFileReference, InferenceRequestIdentity, LanguagePolicy,
                SpeechTranscribeParams, SpeechTranscriptArtifact,
            },
        },
        local_runtime::{admission::Priority, speech},
    },
    local_resources::preparation,
};

const MIN_SPEECH_PERCENT: u64 = 60;
const MAX_TEXT_DISTANCE_PERCENT: usize = 20;
const MAX_MATCH_CHARACTERS: usize = 3_000;

pub(super) async fn validate<R: Runtime>(
    app: &tauri::AppHandle<R>,
    job_id: &str,
    language: &str,
    reference: AudioFileReference,
    confirmed_text: Option<&str>,
    cancellation: &CancellationToken,
) -> Result<String, String> {
    let model = crate::local_resources::model_selection::transcription(app)?;
    let profile = speech::transcription_profile(model)?;
    let reader = hound::WavReader::open(&reference.path).map_err(|error| error.to_string())?;
    let expected_duration_ms = u64::from(reader.duration()) * 1_000 / 16_000;
    drop(reader);
    let resources =
        preparation::prepare_models(app, model.transcription_models(), cancellation, &mut |_| {})
            .await
            .map_err(|error| {
                format!("Voice reference speech recognition is unavailable: {error}")
            })?;
    let request_id = uuid::Uuid::new_v4().to_string();
    let artifact = speech::transcribe(
        app,
        &resources,
        RuntimeDeviceProfile {
            device: profile.device.into(),
            execution_profile_id: profile.id.into(),
            cpu_threads: ComputeBudget::detect()?.model_threads,
        },
        SpeechTranscribeParams {
            identity: InferenceRequestIdentity {
                request_id: request_id.clone(),
                owner_kind: "voice_reference_validation".into(),
                owner_id: job_id.into(),
                attempt_id: job_id.into(),
                attempt_epoch: 1,
                input_revision: format!("sha256:{}", reference.sha256),
                model_spec_id: profile.id.into(),
                deadline_ms: deadline_ms()?,
            },
            audio_ref: reference,
            profile: AsrProfile {
                engine: profile.engine,
                language_policy: LanguagePolicy::Detect,
                language: None,
                device: profile.device.into(),
                compute_type: profile.compute_type.into(),
                beam_size: 5,
                vad_enabled: true,
            },
        },
        cancellation,
        Priority::Interactive,
        |_| {},
    )
    .await?;
    if artifact.duration_ms.abs_diff(expected_duration_ms) > 1 {
        return Err(
            "invalid_output: Voice reference analysis duration differs from its audio".into(),
        );
    }
    validate_artifact(&artifact, language)?;
    if let Some(expected) = confirmed_text {
        validate_confirmed_text(expected, &artifact)?;
    }
    Ok(artifact
        .segments
        .iter()
        .map(|segment| segment.text.trim())
        .collect::<Vec<_>>()
        .join(" "))
}

fn validate_artifact(
    artifact: &SpeechTranscriptArtifact,
    expected_language: &str,
) -> Result<(), String> {
    if !(5_000..=30_000).contains(&artifact.duration_ms)
        || artifact.speech_duration_ms > artifact.duration_ms
    {
        return Err("Voice reference speech analysis returned invalid duration data".into());
    }
    if artifact.speech_duration_ms.saturating_mul(100)
        < artifact.duration_ms.saturating_mul(MIN_SPEECH_PERCENT)
    {
        return Err("Voice reference must contain speech for at least 60% of its duration".into());
    }
    let language_matches = match expected_language {
        "zh-CN" => artifact.language == "zh" || artifact.language == "zh-CN",
        "en" => artifact.language == "en",
        _ => false,
    };
    if !language_matches {
        return Err(format!(
            "Voice reference language was detected as '{}' instead of {expected_language}",
            artifact.language
        ));
    }
    Ok(())
}

fn validate_confirmed_text(
    expected: &str,
    artifact: &SpeechTranscriptArtifact,
) -> Result<(), String> {
    let expected = normalized_characters(expected);
    let actual = normalized_characters(
        &artifact
            .segments
            .iter()
            .map(|segment| segment.text.as_str())
            .collect::<String>(),
    );
    if expected.is_empty()
        || actual.is_empty()
        || expected.len() > MAX_MATCH_CHARACTERS
        || actual.len() > MAX_MATCH_CHARACTERS
    {
        return Err("Confirmed reference transcript could not be matched to audible speech".into());
    }
    let length = expected.len().max(actual.len());
    if edit_distance(&expected, &actual).saturating_mul(100)
        > length.saturating_mul(MAX_TEXT_DISTANCE_PERCENT)
    {
        return Err(
            "Confirmed reference transcript differs from the detected speech by more than 20%"
                .into(),
        );
    }
    Ok(())
}

fn normalized_characters(value: &str) -> Vec<char> {
    value
        .chars()
        .filter(|character| character.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

fn edit_distance(left: &[char], right: &[char]) -> usize {
    let mut previous = (0..=right.len()).collect::<Vec<_>>();
    let mut current = vec![0; right.len() + 1];
    for (left_index, left_character) in left.iter().enumerate() {
        current[0] = left_index + 1;
        for (right_index, right_character) in right.iter().enumerate() {
            current[right_index + 1] = (current[right_index] + 1)
                .min(previous[right_index + 1] + 1)
                .min(previous[right_index] + usize::from(left_character != right_character));
        }
        std::mem::swap(&mut previous, &mut current);
    }
    previous[right.len()]
}

fn deadline_ms() -> Result<u64, String> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| error.to_string())?;
    u64::try_from(now.as_millis())
        .map_err(|_| "Clock exceeds speech protocol range".to_owned())?
        .checked_add(120_000)
        .ok_or_else(|| "Voice reference deadline exceeds speech protocol range".to_owned())
}

#[cfg(test)]
mod tests;

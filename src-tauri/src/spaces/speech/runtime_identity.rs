use std::collections::BTreeMap;

use crate::spaces::db;

pub(in crate::spaces) type RuntimeDigests = BTreeMap<String, String>;

pub(in crate::spaces) fn profile_for_engine(engine: &str) -> Result<String, String> {
    if matches!(engine, "moss_nano" | "voxcpm2") {
        Ok("native-inference-v2".into())
    } else {
        Err("runtime_mismatch: speech engine has no native runtime".into())
    }
}

pub(super) async fn resolve(
    app: &tauri::AppHandle,
    pool: &sqlx::SqlitePool,
    project_id: &str,
    job_id: &str,
    cancellation: &tokio_util::sync::CancellationToken,
) -> Result<RuntimeDigests, String> {
    let engines: Vec<String> = sqlx::query_scalar(
        "SELECT DISTINCT v.engine_id FROM spaces_speech_projects p \
         JOIN spaces_speech_segments s ON s.project_revision_id=p.active_revision_id \
         JOIN spaces_voice_revisions v ON v.id=s.voice_revision_id WHERE p.id=?",
    )
    .bind(project_id)
    .fetch_all(pool)
    .await
    .map_err(db::sql_error)?;
    let mut digests = BTreeMap::new();
    for engine in engines {
        let target = match engine.as_str() {
            "moss_nano" => crate::local_resources::preparation::contracts::ResourcePreparationTarget::SpeechSynthesis,
            "voxcpm2" => crate::local_resources::preparation::contracts::ResourcePreparationTarget::VoiceDesign,
            _ => return Err("runtime_mismatch: speech engine has no resource profile".into()),
        };
        crate::local_resources::preparation::prepare_models_for_job(
            app,
            target,
            &format!("{job_id}:resources:{engine}"),
            "Preparing speech resources",
            cancellation,
            &mut |_| {},
        )
        .await?;
        let profile = profile_for_engine(&engine)?;
        digests.insert(
            profile,
            crate::local_resources::runtimes::inference::resolve(app)?.digest,
        );
    }
    validate(&digests)?;
    Ok(digests)
}

pub(in crate::spaces) fn read_snapshot(
    value: &serde_json::Value,
) -> Result<RuntimeDigests, String> {
    let digests = serde_json::from_value(
        value
            .get("runtimeDigests")
            .cloned()
            .ok_or("runtime_mismatch: Speech Run has no frozen runtime packages")?,
    )
    .map_err(db::json_error)?;
    validate(&digests)?;
    Ok(digests)
}

pub(super) fn validate(digests: &RuntimeDigests) -> Result<(), String> {
    if digests.is_empty()
        || digests
            .iter()
            .any(|(profile, digest)| profile != "native-inference-v2" || !valid_digest(digest))
    {
        return Err("runtime_mismatch: Speech Run runtime package identities are invalid".into());
    }
    Ok(())
}

fn valid_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[cfg(test)]
#[path = "tests/runtime_identity.rs"]
mod tests;

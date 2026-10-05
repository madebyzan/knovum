use super::super::{
    contracts::SpeechSegmentRenderOutput,
    video_config::{self, VideoDubbingOutputConfig},
};
use super::contracts::VideoAudioTrack;
use crate::spaces::db;

#[derive(sqlx::FromRow)]
pub(super) struct Revision {
    pub output_config: String,
    pub output_asset_path: Option<String>,
    pub output_sha256: Option<String>,
}
impl Revision {
    pub fn config(&self) -> Result<VideoDubbingOutputConfig, String> {
        video_config::parse(&serde_json::from_str(&self.output_config).map_err(db::json_error)?)
    }
    pub fn published_output(&self) -> Result<(&str, &str), String> {
        self.output_asset_path
            .as_deref()
            .zip(self.output_sha256.as_deref())
            .ok_or_else(|| "Generate this dubbing revision before exporting its audio".into())
    }
}
pub(super) async fn load(
    pool: &sqlx::SqlitePool,
    project_id: &str,
    revision_id: &str,
) -> Result<Revision, String> {
    sqlx::query_as("SELECT r.output_config,r.output_asset_path,r.output_sha256 FROM spaces_speech_project_revisions r JOIN spaces_speech_projects p ON p.id=r.project_id WHERE p.id=? AND r.id=? AND p.project_type='video_dubbing'")
        .bind(project_id).bind(revision_id).fetch_optional(pool).await.map_err(db::sql_error)?
        .ok_or_else(|| "Video dubbing revision is unavailable".into())
}
pub(super) async fn tracks(
    pool: &sqlx::SqlitePool,
    project_id: &str,
) -> Result<Vec<VideoAudioTrack>, String> {
    let rows: Vec<(String, i64, String)> = sqlx::query_as("SELECT r.id,r.revision,r.output_config FROM spaces_speech_project_revisions r JOIN spaces_speech_projects p ON p.id=r.project_id WHERE p.id=? AND p.project_type='video_dubbing' AND r.output_asset_path IS NOT NULL ORDER BY r.revision DESC")
        .bind(project_id).fetch_all(pool).await.map_err(db::sql_error)?;
    let mut tracks = Vec::with_capacity(rows.len());
    for (revision_id, revision, output_config) in rows {
        let config =
            video_config::parse(&serde_json::from_str(&output_config).map_err(db::json_error)?)?;
        let languages: Vec<String> = sqlx::query_scalar(
            "SELECT DISTINCT language FROM spaces_speech_segments WHERE project_revision_id=?",
        )
        .bind(&revision_id)
        .fetch_all(pool)
        .await
        .map_err(db::sql_error)?;
        let language = if let Some(translation) = config.translation {
            translation.target_language
        } else if languages.len() == 1 {
            languages[0].clone()
        } else {
            "und".into()
        };
        tracks.push(VideoAudioTrack {
            revision_id,
            revision,
            language,
        });
    }
    Ok(tracks)
}
#[derive(sqlx::FromRow)]
struct Render {
    segment_id: String,
    path: String,
    sha256: String,
    sample_rate: i64,
    sample_count: i64,
}
pub(super) async fn renders(
    pool: &sqlx::SqlitePool,
    revision_id: &str,
) -> Result<Vec<SpeechSegmentRenderOutput>, String> {
    let rows: Vec<Render> = sqlx::query_as("SELECT s.id AS segment_id,r.asset_path AS path,r.output_sha256 AS sha256,r.sample_rate,r.sample_count FROM spaces_speech_segments s JOIN spaces_speech_renders r ON r.segment_id=s.id WHERE s.project_revision_id=? AND r.state='published' AND r.attempt_epoch=(SELECT MAX(r2.attempt_epoch) FROM spaces_speech_renders r2 WHERE r2.segment_id=s.id AND r2.state='published') ORDER BY s.ordinal")
        .bind(revision_id).fetch_all(pool).await.map_err(db::sql_error)?;
    rows.into_iter()
        .map(|row| {
            Ok(SpeechSegmentRenderOutput {
                segment_id: row.segment_id,
                path: row.path,
                sha256: row.sha256,
                sample_rate: u32::try_from(row.sample_rate)
                    .map_err(|_| "Speech render sample rate is invalid")?,
                sample_count: u64::try_from(row.sample_count)
                    .map_err(|_| "Speech render sample count is invalid")?,
                cached: true,
            })
        })
        .collect()
}

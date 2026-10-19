use crate::spaces::speech::{contracts::GenerateSpeechProjectRequest, run_start, steps};

/// Every planned step must be readable by the executor that claims its kind,
/// otherwise the Run only fails after the user already waited for it.
#[tokio::test]
async fn planned_steps_deserialize_into_the_executor_configuration_of_their_kind() {
    for (project_type, segments, expected_steps) in [
        (
            "short_dubbing",
            &[("moss_nano", None), ("voxcpm2", None)][..],
            3,
        ),
        (
            "audiobook",
            &[
                ("moss_nano", Some("one")),
                ("moss_nano", Some("one")),
                ("voxcpm2", Some("two")),
            ][..],
            7,
        ),
    ] {
        let pool = super::database().await;
        let project_id = super::speech_project(&pool, project_type, segments).await;
        let request_id = uuid::Uuid::new_v4().to_string();
        let mut digests = crate::spaces::speech::runtime_identity::RuntimeDigests::new();
        digests.insert("native-inference-v2".into(), "b".repeat(64));
        run_start::start(
            &pool,
            GenerateSpeechProjectRequest {
                request_id: request_id.clone(),
                project_id,
                expected_revision: 1,
            },
            &digests,
            &super::reservation_context(),
        )
        .await
        .unwrap();
        let planned: Vec<(String, String)> =
            sqlx::query_as("SELECT node_kind,configuration FROM spaces_run_steps WHERE run_id=?")
                .bind(&request_id)
                .fetch_all(&pool)
                .await
                .unwrap();
        assert_eq!(planned.len(), expected_steps);
        for (kind, configuration) in planned {
            let configuration = serde_json::from_str(&configuration).unwrap();
            readable(&kind, &configuration)
                .unwrap_or_else(|error| panic!("{kind} configuration is unreadable: {error}"));
        }
    }
}

fn readable(kind: &str, configuration: &serde_json::Value) -> Result<(), String> {
    match kind {
        "speech_segment" => steps::parse::<steps::SegmentStep>(configuration).map(|_| ()),
        "speech_chapter_pcm" => steps::parse::<steps::ChapterStep>(configuration).map(|_| ()),
        "speech_loudness_measurement" | "speech_audiobook_encode" => {
            steps::parse::<steps::RevisionStep>(configuration).map(|_| ())
        }
        "speech_assembly" => steps::parse::<steps::AssemblyStep>(configuration).map(|_| ()),
        other => panic!("speech planning produced an unroutable step kind: {other}"),
    }
}

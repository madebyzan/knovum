use super::super::{
    execution::is_publication_for_claim,
    media::SAMPLE_RATE,
    repository::{commit_publication, list_publications_for_source, load_publication},
};
use super::{database, source};
use crate::{
    ai::local_runtime::separation::{SeparatedAudio, SeparatedAudioMetadata},
    spaces::runs::AttemptClaim,
};
use serde_json::json;
use std::path::Path;
use tokio_util::sync::CancellationToken;

const MODEL_SPEC_ID: &str = "spleeter-2stems-fp16-cpu-v1";

#[tokio::test]
async fn publication_commit_requires_a_live_attempt_and_publishes_two_related_nodes_atomically() {
    let pool = database().await;
    let run_id = uuid::Uuid::new_v4().to_string();
    let step_id = uuid::Uuid::new_v4().to_string();
    let attempt_id = uuid::Uuid::new_v4().to_string();
    let now = crate::clock::unix_time_millis().unwrap();
    sqlx::query("INSERT INTO spaces_runs(id,scope_kind,home_space_id,principal_kind,principal_id,kind,state,execution_epoch,policy_snapshot,input_snapshot,budget,created_at,updated_at) VALUES(?,'space','space','user','local-user','workflow','running',2,'{}','{}','{}',1,1)")
        .bind(&run_id).execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO spaces_run_steps(id,run_id,definition_node_id,instance_path,node_kind,state,resolved_input,created_at,updated_at) VALUES(? ,?,'separate','separate','speech_source_separation','running','{}',1,1)")
        .bind(&step_id).bind(&run_id).execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO spaces_run_attempts(id,run_id,step_instance_id,attempt_number,state,request_digest,executor_id,idempotency_key,execution_epoch,lease_until,started_at) VALUES(?,?,?,1,'executing',?,'executor','key',2,?,1)")
        .bind(&attempt_id).bind(&run_id).bind(&step_id).bind("b".repeat(64)).bind(now - 1)
        .execute(&pool).await.unwrap();
    let claim = AttemptClaim {
        attempt_id: attempt_id.clone(),
        run_id: run_id.clone(),
        step_instance_id: step_id.clone(),
        execution_epoch: 2,
        attempt_number: 1,
        scope: crate::spaces::authorization::RunScope::Space {
            space_id: "space".into(),
        },
        node_kind: "speech_source_separation".into(),
        configuration: json!({"sourceNodeId":"media","vocalsLabel":"Vocals","backgroundLabel":"Background","source":source(),"model":"spleeter2Stems"}),
        resolved_input: json!({}),
        policy_snapshot: json!({}),
        budget: json!({}),
        cancellation: CancellationToken::new(),
    };
    let background = Path::new("/managed/background.wav");
    let vocals = Path::new("/managed/vocals.wav");
    let metadata = SeparatedAudio {
        background: SeparatedAudioMetadata {
            sha256: "c".repeat(64),
            sample_rate: SAMPLE_RATE,
            sample_count: SAMPLE_RATE.into(),
        },
        vocals: SeparatedAudioMetadata {
            sha256: "e".repeat(64),
            sample_rate: SAMPLE_RATE,
            sample_count: SAMPLE_RATE.into(),
        },
    };
    let error = commit_publication(
        &pool,
        &claim,
        &source(),
        background,
        vocals,
        &metadata,
        1_000,
    )
    .await
    .unwrap_err();
    assert!(error.contains("attempt-stale"));

    sqlx::query("UPDATE spaces_run_attempts SET lease_until=? WHERE id=?")
        .bind(now + 60_000)
        .bind(&attempt_id)
        .execute(&pool)
        .await
        .unwrap();
    crate::spaces::speech::asset_publication::stage_batch_for_attempt(&pool, &claim, &[
        crate::spaces::speech::asset_publication::PendingAsset { owner_kind: crate::spaces::speech::asset_publication::PublicationOwnerKind::SourceSeparation, owner_id: &run_id, asset_path: background, part_path: None, sha256: &metadata.background.sha256 },
        crate::spaces::speech::asset_publication::PendingAsset { owner_kind: crate::spaces::speech::asset_publication::PublicationOwnerKind::SourceSeparation, owner_id: &run_id, asset_path: vocals, part_path: None, sha256: &metadata.vocals.sha256 },
    ]).await.unwrap();
    {
        // An attempt can lose its lease while waiting for another writer's connection.
        let mut writer = crate::spaces::db::begin_write(&pool).await.unwrap();
        let lease_until = crate::clock::unix_time_millis().unwrap() + 50;
        sqlx::query("UPDATE spaces_run_attempts SET lease_until=? WHERE id=?")
            .bind(lease_until)
            .bind(&attempt_id)
            .execute(&mut *writer)
            .await
            .unwrap();
        let pending_source = source();
        let pending = commit_publication(
            &pool,
            &claim,
            &pending_source,
            background,
            vocals,
            &metadata,
            1_000,
        );
        tokio::pin!(pending);
        std::future::poll_fn(|context| {
            assert!(std::future::Future::poll(pending.as_mut(), context).is_pending());
            std::task::Poll::Ready(())
        })
        .await;
        while crate::clock::unix_time_millis().unwrap() <= lease_until {
            tokio::time::sleep(std::time::Duration::from_millis(1)).await;
        }
        writer.commit().await.unwrap();
        assert!(pending.await.unwrap_err().contains("attempt-stale"));
    }
    assert!(super::super::repository::find_publication(&pool, &run_id)
        .await
        .unwrap()
        .is_none());
    sqlx::query("UPDATE spaces_run_attempts SET lease_until=? WHERE id=?")
        .bind(crate::clock::unix_time_millis().unwrap() + 60_000)
        .bind(&attempt_id)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("DELETE FROM spaces_speech_asset_publications WHERE asset_path=?")
        .bind(vocals.to_string_lossy().as_ref())
        .execute(&pool)
        .await
        .unwrap();
    let error = commit_publication(
        &pool,
        &claim,
        &source(),
        background,
        vocals,
        &metadata,
        1_000,
    )
    .await
    .unwrap_err();
    assert!(error.contains("Asset publication ownership record is missing"));
    assert!(super::super::repository::find_publication(&pool, &run_id)
        .await
        .unwrap()
        .is_none());
    let nodes: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM spaces_nodes WHERE kind='audio'")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(nodes, 0, "a missing stem journal must roll back both nodes");
    let relations: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM spaces_relations")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(relations, 0);
    let remaining: i64 =
        sqlx::query_scalar("SELECT count(*) FROM spaces_speech_asset_publications")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(
        remaining, 1,
        "a failed two-track commit must retain the first recovery journal"
    );
    crate::spaces::speech::asset_publication::stage_for_attempt(
        &pool,
        &claim,
        crate::spaces::speech::asset_publication::PendingAsset {
            owner_kind:
                crate::spaces::speech::asset_publication::PublicationOwnerKind::SourceSeparation,
            owner_id: &run_id,
            asset_path: vocals,
            part_path: None,
            sha256: &metadata.vocals.sha256,
        },
    )
    .await
    .unwrap();
    sqlx::query(
        "UPDATE spaces_nodes SET data=json_set(data,'$.sourceRevision',?) WHERE id='media'",
    )
    .bind(format!("sha256:{}", "f".repeat(64)))
    .execute(&pool)
    .await
    .unwrap();
    let stale = commit_publication(
        &pool,
        &claim,
        &source(),
        background,
        vocals,
        &metadata,
        1_000,
    )
    .await
    .unwrap_err();
    assert!(stale.contains("stale_revision"));
    assert!(super::super::repository::find_publication(&pool, &run_id)
        .await
        .unwrap()
        .is_none());
    sqlx::query(
        "UPDATE spaces_nodes SET data=json_set(data,'$.sourceRevision',?) WHERE id='media'",
    )
    .bind(source().source_revision)
    .execute(&pool)
    .await
    .unwrap();
    let mut cancelled = claim.clone();
    cancelled.cancellation = CancellationToken::new();
    cancelled.cancellation.cancel();
    assert_eq!(
        commit_publication(
            &pool,
            &cancelled,
            &source(),
            background,
            vocals,
            &metadata,
            1_000
        )
        .await
        .unwrap_err(),
        "cancelled"
    );
    commit_publication(
        &pool,
        &claim,
        &source(),
        background,
        vocals,
        &metadata,
        1_000,
    )
    .await
    .unwrap();

    let publication = load_publication(&pool, &run_id).await.unwrap();
    let space_id = claim.scope.require_space_id().unwrap().to_owned();
    assert!(is_publication_for_claim(
        &claim,
        &space_id,
        &source(),
        &publication
    ));
    assert_eq!(publication.vocals.asset_path, vocals.to_string_lossy());
    assert_eq!(
        publication.vocals.source_revision,
        format!("sha256:{}", metadata.vocals.sha256)
    );
    assert_eq!(
        publication.vocals.duration_ms,
        publication.background.duration_ms
    );
    let pending: i64 = sqlx::query_scalar("SELECT count(*) FROM spaces_speech_asset_publications")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(pending, 0);
    let node_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM spaces_nodes WHERE kind='audio'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(
        node_count, 2,
        "both stems must be published without review or insertion clicks"
    );
    let relations: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM spaces_relations WHERE kind='derives-from'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(relations, 2);
    let sources: Vec<String> =
        sqlx::query_scalar("SELECT target_id FROM spaces_relations ORDER BY source_id")
            .fetch_all(&pool)
            .await
            .unwrap();
    assert_eq!(sources, vec!["media", "media"]);
    let titles: Vec<String> = sqlx::query_scalar(
        "SELECT json_extract(data,'$.title') FROM spaces_nodes WHERE kind='audio' ORDER BY data",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert!(titles.contains(&"Interview · Vocals".into()));
    assert!(titles.contains(&"Interview · Background".into()));

    let published = list_publications_for_source(&pool, "space", &source())
        .await
        .unwrap();
    assert_eq!(published.len(), 1);
    assert_eq!(published[0].separation_id, run_id);
    assert!(list_publications_for_source(&pool, "other", &source())
        .await
        .unwrap()
        .is_empty());
    let mut changed_source = source();
    changed_source.source_revision = format!("sha256:{}", "d".repeat(64));
    assert!(
        list_publications_for_source(&pool, "space", &changed_source)
            .await
            .unwrap()
            .is_empty()
    );
    assert!(!is_publication_for_claim(
        &claim,
        &space_id,
        &changed_source,
        &publication
    ));
    assert_eq!(
        load_publication(&pool, &run_id)
            .await
            .unwrap()
            .model_spec_id,
        MODEL_SPEC_ID
    );
    let mut changed_claim = claim;
    changed_claim.configuration = json!({"sourceNodeId":"media","vocalsLabel":"Vocals","backgroundLabel":"Background","source":source(), "model":"spleeter2StemsFull"});
    assert!(!is_publication_for_claim(
        &changed_claim,
        &space_id,
        &source(),
        &publication
    ));
    let mut full_publication = publication;
    full_publication.model_spec_id =
        crate::local_resources::model_catalog::LocalModelId::Spleeter2StemsFull
            .separation_spec()
            .unwrap()
            .into();
    assert!(is_publication_for_claim(
        &changed_claim,
        &space_id,
        &source(),
        &full_publication
    ));
    changed_claim.configuration = json!({"sourceNodeId":"media","vocalsLabel":"Vocals","backgroundLabel":"Background","source":source()});
    assert!(!is_publication_for_claim(
        &changed_claim,
        &space_id,
        &source(),
        &full_publication
    ));
}

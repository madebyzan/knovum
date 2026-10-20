use sqlx::sqlite::SqlitePoolOptions;

#[tokio::test]
async fn image_object_removal_is_a_valid_derivation_action() {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("database");
    super::migrate(&pool).await.expect("baseline");
    sqlx::query("INSERT INTO spaces(id,kind,title,created_at,updated_at) VALUES('space','space','Space',1,1)")
        .execute(&pool).await.expect("space");
    sqlx::query("INSERT INTO spaces_nodes(id,space_id,kind,initial_x,initial_y,data,created_at,updated_at) VALUES('output','space','image',0,0,'{\"kind\":\"image\",\"title\":\"Output\"}',1,1)")
        .execute(&pool).await.expect("image output");
    sqlx::query("INSERT INTO spaces_knowledge_derivations(id,space_id,output_node_id,action_kind,action_version,engine,status,created_at,updated_at) VALUES('job','space','output','remove-image-object',1,'dashscope:wanx2.1-imageedit:mask','running',1,1)")
        .execute(&pool).await.expect("object-removal derivation");
}

#[tokio::test]
async fn obsolete_migration_history_is_rejected_without_losing_user_rows() {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("database");
    super::migrate(&pool).await.expect("baseline");
    sqlx::query(
        "INSERT INTO spaces(id,kind,title,created_at,updated_at) VALUES('keep','space','Keep',1,1)",
    )
    .execute(&pool)
    .await
    .expect("user row");
    sqlx::query("INSERT INTO _sqlx_migrations(version,description,success,checksum,execution_time) VALUES(2,'obsolete',1,x'00',0)").execute(&pool).await.expect("old version");
    let error = super::migrate(&pool)
        .await
        .expect_err("obsolete history must fail");
    assert!(matches!(
        error,
        sqlx::migrate::MigrateError::VersionMissing(2)
    ));
    let title: String = sqlx::query_scalar("SELECT title FROM spaces WHERE id='keep'")
        .fetch_one(&pool)
        .await
        .expect("original data preserved");
    assert_eq!(title, "Keep");
}

#[tokio::test]
async fn obsolete_baseline_checksum_is_rejected_without_resetting_the_database() {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("database");
    super::migrate(&pool).await.expect("baseline");
    sqlx::query(
        "INSERT INTO spaces(id,kind,title,created_at,updated_at) VALUES('keep','space','Keep',1,1)",
    )
    .execute(&pool)
    .await
    .expect("user row");
    sqlx::query("UPDATE _sqlx_migrations SET checksum=x'00' WHERE version=1")
        .execute(&pool)
        .await
        .expect("obsolete baseline");
    assert!(matches!(
        super::migrate(&pool).await,
        Err(sqlx::migrate::MigrateError::VersionMismatch(1))
    ));
    let title: String = sqlx::query_scalar("SELECT title FROM spaces WHERE id='keep'")
        .fetch_one(&pool)
        .await
        .expect("original data preserved");
    assert_eq!(title, "Keep");
}

#[tokio::test]
async fn diarization_requests_require_and_preserve_their_frozen_model() {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    super::migrate(&pool).await.unwrap();
    sqlx::query("INSERT INTO spaces(id,kind,title,created_at,updated_at) VALUES('space','space','Space',1,1)").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO spaces_nodes(id,space_id,kind,initial_x,initial_y,data,created_at,updated_at) VALUES('audio','space','audio',0,0,'{\"kind\":\"audio\",\"title\":\"Audio\"}',1,1)").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO spaces_enrichment_jobs(id,space_id,node_id,job_type,status,source_revision,created_at,updated_at) VALUES('job','space','audio','diarize_media_transcript','pending','revision',1,1)").execute(&pool).await.unwrap();
    assert!(sqlx::query("INSERT INTO spaces_media_diarization_requests(job_id,expected_transcript_revision) VALUES('job',1)").execute(&pool).await.is_err());
    let model = crate::local_resources::model_catalog::LocalModelId::SherpaDiarizationFull;
    let spec = model.diarization_variant().unwrap().spec_id();
    sqlx::query("INSERT INTO spaces_media_diarization_requests(job_id,model_spec_id,expected_transcript_revision) VALUES('job',$1,1)").bind(spec).execute(&pool).await.unwrap();
    sqlx::query("UPDATE spaces_enrichment_jobs SET attempts=2,status='running' WHERE id='job'")
        .execute(&pool)
        .await
        .unwrap();
    let stored: String = sqlx::query_scalar(
        "SELECT model_spec_id FROM spaces_media_diarization_requests WHERE job_id='job'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        crate::local_resources::model_catalog::LocalModelId::from_diarization_spec(&stored)
            .unwrap(),
        model
    );
}

#[tokio::test]
async fn alignment_requests_require_the_model_before_scheduling() {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    super::migrate(&pool).await.unwrap();
    sqlx::query("INSERT INTO spaces(id,kind,title,created_at,updated_at) VALUES('space','space','Space',1,1)").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO spaces_nodes(id,space_id,kind,initial_x,initial_y,data,created_at,updated_at) VALUES('audio','space','audio',0,0,'{\"kind\":\"audio\",\"title\":\"Audio\"}',1,1)").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO spaces_enrichment_jobs(id,space_id,node_id,job_type,status,source_revision,created_at,updated_at) VALUES('job','space','audio','align_media_transcript','pending','revision',1,1)").execute(&pool).await.unwrap();
    assert!(sqlx::query("INSERT INTO spaces_media_alignment_requests(job_id,expected_transcript_revision,segment_ids) VALUES('job',1,'[\"segment\"]')").execute(&pool).await.is_err());
    let model = crate::local_resources::model_catalog::LocalModelId::Qwen3AlignerQ8;
    sqlx::query("INSERT INTO spaces_media_alignment_requests(job_id,model_spec_id,expected_transcript_revision,segment_ids) VALUES('job',$1,1,'[\"segment\"]')")
        .bind(model.alignment_spec().unwrap()).execute(&pool).await.unwrap();
    let spec: String = sqlx::query_scalar(
        "SELECT model_spec_id FROM spaces_media_alignment_requests WHERE job_id='job'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        crate::local_resources::model_catalog::LocalModelId::from_alignment_spec(&spec).unwrap(),
        model
    );
}

#[tokio::test]
async fn speech_schema_accepts_moss_reference_and_full_separation() {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    super::migrate(&pool).await.unwrap();
    sqlx::query(
        "INSERT INTO spaces(id,kind,title,created_at,updated_at) VALUES('space','space','Space',1,1)",
    )
    .execute(&pool)
    .await
    .unwrap();
    for run_id in ["moss-reference", "moss-designed"] {
        sqlx::query("INSERT INTO spaces_runs(id,scope_kind,home_space_id,principal_kind,principal_id,kind,state,policy_snapshot,input_snapshot,budget,created_at,updated_at,completed_at) VALUES(?,'application',NULL,'user','local-user','workflow','succeeded','{}','{\"workflow\":\"speech_voice_candidate_v1\"}','{}',1,1,1)")
            .bind(run_id).execute(&pool).await.unwrap();
    }
    sqlx::query("INSERT INTO spaces_runs(id,scope_kind,home_space_id,principal_kind,principal_id,kind,state,policy_snapshot,input_snapshot,budget,created_at,updated_at) VALUES('full-separation','space','space','user','local-user','workflow','running','{}','{}','{}',1,1)")
        .execute(&pool).await.unwrap();

    sqlx::query(
        "INSERT INTO spaces_voice_candidates( \
         id,run_id,source_kind,language,sample_text,design_description,reference_text, \
         input_source_revision,input_byte_length,reference_asset_path,reference_sha256,reference_byte_length, \
         preview_asset_path,preview_sha256,preview_sample_rate,preview_sample_count,model_spec_id,seed, \
         synthesis_config,authorization,status,created_at) \
         VALUES('moss-reference','moss-reference','reference','en','Sample',NULL,NULL,?,32044, \
         '/moss-reference.wav',?,32044,'/moss-preview.wav',?,48000,48000, \
         'moss-tts-nano-100m-onnx-bundled-v2',0,?,?,'ready',1)",
    )
    .bind(format!("sha256:{}", "a".repeat(64)))
    .bind("b".repeat(64))
    .bind("c".repeat(64))
    .bind(r#"{"speed":1.0,"sampleRate":48000,"seed":0}"#)
    .bind(r#"{"declared":true,"referenceTranscriptConfirmed":false}"#)
    .execute(&pool)
    .await
    .unwrap();

    let obsolete = sqlx::query("UPDATE spaces_voice_candidates SET model_spec_id='moss-tts-nano-onnx-v1' WHERE id='moss-reference'")
        .execute(&pool).await;
    assert!(obsolete.is_err());

    let designed_moss = sqlx::query(
        "INSERT INTO spaces_voice_candidates( \
         id,run_id,source_kind,language,sample_text,design_description,reference_text, \
         reference_asset_path,reference_sha256,reference_byte_length,preview_asset_path,preview_sha256, \
         preview_sample_rate,preview_sample_count,model_spec_id,seed,synthesis_config,authorization,status,created_at) \
         VALUES('moss-designed','moss-designed','designed','en','Sample','Warm','Sample', \
         '/moss-designed.wav',?,32044,'/moss-designed-preview.wav',?,48000,48000, \
         'moss-tts-nano-100m-onnx-bundled-v2',0,?,?,'ready',1)",
    )
    .bind("d".repeat(64))
    .bind("e".repeat(64))
    .bind(r#"{"speed":1.0,"sampleRate":48000,"seed":0}"#)
    .bind(r#"{"declared":true,"referenceTranscriptConfirmed":true}"#)
    .execute(&pool)
    .await;
    assert!(designed_moss.is_err());

    sqlx::query("INSERT INTO spaces_nodes(id,space_id,kind,initial_x,initial_y,data,created_at,updated_at) VALUES('source','space','audio',0,0,'{\"kind\":\"audio\",\"title\":\"Source\"}',1,1)")
        .execute(&pool).await.unwrap();
    sqlx::query(
        "INSERT INTO spaces_speech_source_separations( \
         id,space_id,run_id,source_node_id,source_asset_path,source_revision,source_duration_ms,model_spec_id, \
         background_asset_path,vocals_asset_path,vocals_sha256,background_sha256,background_sample_rate, \
         background_sample_count,background_duration_ms,created_at) \
         VALUES('full-separation','space','full-separation','source','/source.wav',?,1000, \
         'spleeter-2stems-fp32-cpu-v1','/background.wav','/vocals.wav',?,?,44100,44100,1000, \
         1)",
    )
    .bind(format!("sha256:{}", "f".repeat(64)))
    .bind("1".repeat(64))
    .bind("2".repeat(64))
    .execute(&pool)
    .await
    .unwrap();
}

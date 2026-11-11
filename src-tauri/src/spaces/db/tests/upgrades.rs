//! Each migration upgrades the schema the previous release shipped. A test here starts from that
//! schema, carries representative rows through the migration, and checks that the upgraded library
//! is the one a fresh install creates, with the rows it had.

use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

/// A library exactly as `version` left it.
async fn shipped(version: i64) -> SqlitePool {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("database");
    super::super::MIGRATOR
        .run_to(version, &pool)
        .await
        .expect("shipped schema");
    pool
}

async fn schema(pool: &SqlitePool) -> Vec<String> {
    sqlx::query_scalar("SELECT sql FROM sqlite_master WHERE sql IS NOT NULL ORDER BY name")
        .fetch_all(pool)
        .await
        .expect("schema")
}

/// 0.0.1 recorded the sessions of the removed Create with Agent dialog. Upgrading settles the Run
/// a crash left open — nothing can finish it any more — and keeps every Run, while the session
/// table leaves with the feature.
#[tokio::test]
async fn upgrading_0_0_1_settles_and_retires_agent_artifact_sessions() {
    let pool = shipped(1).await;
    sqlx::query(
        "INSERT INTO spaces(id,kind,title,created_at,updated_at) VALUES('s','space','S',1,1)",
    )
    .execute(&pool)
    .await
    .expect("space");
    for (run, state, completed_at) in [
        ("interrupted", "running", None),
        ("finished", "succeeded", Some(5_i64)),
        ("unrelated", "running", None),
    ] {
        sqlx::query(
            "INSERT INTO spaces_runs(id,scope_kind,home_space_id,principal_kind,principal_id,kind, \
             state,state_version,policy_snapshot,input_snapshot,budget,created_at,updated_at, \
             completed_at) \
             VALUES($1,'space','s','external_agent',$1,'external_agent',$2,1,'{}','{}','{}',1,1,$3)",
        )
        .bind(run)
        .bind(state)
        .bind(completed_at)
        .execute(&pool)
        .await
        .expect("run");
    }
    for (session, state) in [("interrupted", "running"), ("finished", "completed")] {
        sqlx::query(
            "INSERT INTO spaces_agent_artifact_sessions(id,run_id,space_id,runtime_kind,state, \
             created_at,updated_at) VALUES($1,$1,'s','codex',$2,1,1)",
        )
        .bind(session)
        .bind(state)
        .execute(&pool)
        .await
        .expect("session");
    }

    super::super::migrate(&pool).await.expect("upgrade");

    assert_eq!(schema(&pool).await, schema(&super::migrated().await).await);
    let runs: Vec<(String, String, i64, Option<i64>)> =
        sqlx::query_as("SELECT id,state,state_version,completed_at FROM spaces_runs ORDER BY id")
            .fetch_all(&pool)
            .await
            .expect("runs");
    let settled: Vec<_> = runs
        .iter()
        .map(|(id, state, version, completed_at)| {
            (
                id.as_str(),
                state.as_str(),
                *version,
                completed_at.is_some_and(|at| at > 1),
            )
        })
        .collect();
    assert_eq!(
        settled,
        [
            ("finished", "succeeded", 1, true),
            ("interrupted", "failed", 2, true),
            ("unrelated", "running", 1, false),
        ]
    );
}

/// 0.0.3 kept a picture of an attached image node in the message. A node attachment is a
/// reference drawn from the node itself now, so upgrading takes the picture out of every node
/// attachment, in place and in order, and leaves an uploaded file's — the only copy it has.
#[tokio::test]
async fn upgrading_0_0_3_draws_node_attachments_from_their_nodes() {
    let pool = shipped(2).await;
    for statement in [
        "INSERT INTO spaces(id,kind,title,created_at,updated_at) VALUES('s','space','S',1,1)",
        "INSERT INTO spaces_conversations(id,space_id,title,created_at,updated_at) \
         VALUES('c','s','C',1,1)",
    ] {
        sqlx::query(statement)
            .execute(&pool)
            .await
            .expect("fixture");
    }
    let sent = serde_json::json!({"parts": [], "citations": [], "attachments": [
        {"source": "node", "nodeId": "deck", "title": "Deck", "kind": "file", "thumbnail": null},
        {"source": "file", "title": "dog.png", "kind": "image", "thumbnail": "data:image/png;base64,AA"},
        {"source": "node", "nodeId": "dog", "title": "Dog", "kind": "image", "thumbnail": "data:image/png;base64,BB"},
    ]});
    let plain = serde_json::json!({"parts": [{"type": "text", "text": "hi"}], "citations": []});
    for (id, content) in [("sent", &sent), ("plain", &plain)] {
        sqlx::query(
            "INSERT INTO spaces_conversation_messages(id,conversation_id,branch_id,role,content, \
             created_at) VALUES($1,'c','c','user',$2,1)",
        )
        .bind(id)
        .bind(content.to_string())
        .execute(&pool)
        .await
        .expect("message");
    }

    super::super::migrate(&pool).await.expect("upgrade");

    assert_eq!(schema(&pool).await, schema(&super::migrated().await).await);
    let content = |id: &'static str| {
        let pool = pool.clone();
        async move {
            let text: String =
                sqlx::query_scalar("SELECT content FROM spaces_conversation_messages WHERE id=$1")
                    .bind(id)
                    .fetch_one(&pool)
                    .await
                    .expect("content");
            serde_json::from_str::<serde_json::Value>(&text).expect("json")
        }
    };
    assert_eq!(
        content("sent").await["attachments"],
        serde_json::json!([
            {"source": "node", "nodeId": "deck", "title": "Deck", "kind": "file"},
            {"source": "file", "title": "dog.png", "kind": "image", "thumbnail": "data:image/png;base64,AA"},
            {"source": "node", "nodeId": "dog", "title": "Dog", "kind": "image"},
        ])
    );
    assert_eq!(content("plain").await, plain);
}

/// 0.1.0 has no text recognition and stored every PDF picture as its macOS renderer drew it, upside
/// down and small. Upgrading admits the new job, keeps every job with the requests that hang off it,
/// and queues each live PDF to be parsed again, once; Office documents, whose pictures were never
/// rendered, keep theirs.
#[tokio::test]
async fn upgrading_0_1_0_admits_text_recognition_and_rerenders_pdf_pictures() {
    let pool = shipped(3).await;
    sqlx::query(
        "INSERT INTO spaces(id,kind,title,created_at,updated_at) VALUES('s','space','S',1,1)",
    )
    .execute(&pool)
    .await
    .expect("space");
    let file = |parser: &str| {
        serde_json::json!({"kind":"file","title":"Brief","sourceRevision":"r1",
            "documentCoverage":{"parser":parser,"resources":[]}})
    };
    for (id, deleted_at, data) in [
        ("pdf", None, file("pdf-inspector")),
        ("queued", None, file("pdf-inspector")),
        ("trashed", Some(5_i64), file("pdf-inspector")),
        ("deck", None, file("office-oxide")),
        (
            "talk",
            None,
            serde_json::json!({"kind":"audio","title":"Talk","sourceRevision":"a1"}),
        ),
    ] {
        sqlx::query(
            "INSERT INTO spaces_nodes(id,space_id,kind,initial_x,initial_y,data,created_at,updated_at,deleted_at) \
             VALUES($1,'s',$2,0,0,$3,1,1,$4)",
        )
        .bind(id)
        .bind(data["kind"].as_str().expect("kind"))
        .bind(data.to_string())
        .bind(deleted_at)
        .execute(&pool)
        .await
        .expect("node");
    }
    for statement in [
        "INSERT INTO spaces_enrichment_jobs(id,space_id,node_id,job_type,status,source_revision,attempts,created_at,updated_at) \
         VALUES('waiting','s','queued','parse_document','pending','r1',0,1,1)",
        "INSERT INTO spaces_enrichment_jobs(id,space_id,node_id,job_type,status,source_revision,attempts,created_at,updated_at) \
         VALUES('align','s','talk','align_media_transcript','pending','a1',0,1,1)",
        "INSERT INTO spaces_enrichment_jobs(id,space_id,node_id,job_type,status,source_revision,attempts,created_at,updated_at) \
         VALUES('diarize','s','talk','diarize_media_transcript','pending','a1',0,1,1)",
        "INSERT INTO spaces_media_alignment_requests(job_id,model_spec_id,expected_transcript_revision,segment_ids) \
         VALUES('align','aligner',2,'[\"s1\"]')",
        "INSERT INTO spaces_media_diarization_requests(job_id,model_spec_id,expected_transcript_revision) \
         VALUES('diarize','diarizer',2)",
    ] {
        sqlx::query(statement).execute(&pool).await.expect("job");
    }

    super::super::migrate(&pool).await.expect("upgrade");

    assert_eq!(schema(&pool).await, schema(&super::migrated().await).await);
    let requests: (i64, i64) = sqlx::query_as(
        "SELECT (SELECT COUNT(*) FROM spaces_media_alignment_requests WHERE job_id='align' \
           AND segment_ids='[\"s1\"]'), \
         (SELECT COUNT(*) FROM spaces_media_diarization_requests WHERE job_id='diarize')",
    )
    .fetch_one(&pool)
    .await
    .expect("requests");
    assert_eq!(
        requests,
        (1, 1),
        "the rebuilt job table keeps what hangs off it"
    );
    sqlx::query(
        "INSERT INTO spaces_enrichment_jobs(id,space_id,node_id,job_type,status,source_revision,attempts,created_at,updated_at) \
         VALUES('read','s','pdf','recognize_text','pending','r1',0,2,2)",
    )
    .execute(&pool)
    .await
    .expect("text recognition is a job type");
    let covered: Vec<(String, bool)> = sqlx::query_as(
        "SELECT id, json_extract(data,'$.documentCoverage') IS NOT NULL FROM spaces_nodes \
         WHERE kind='file' ORDER BY id",
    )
    .fetch_all(&pool)
    .await
    .expect("nodes");
    assert_eq!(
        covered,
        [
            ("deck".to_owned(), true),
            ("pdf".to_owned(), false),
            ("queued".to_owned(), false),
            ("trashed".to_owned(), false),
        ]
    );
    let parses: Vec<(String, String, String)> = sqlx::query_as(
        "SELECT node_id,status,source_revision FROM spaces_enrichment_jobs \
         WHERE job_type='parse_document' ORDER BY node_id",
    )
    .fetch_all(&pool)
    .await
    .expect("jobs");
    assert_eq!(
        parses,
        [
            ("pdf".to_owned(), "pending".to_owned(), "r1".to_owned()),
            ("queued".to_owned(), "pending".to_owned(), "r1".to_owned()),
        ]
    );
}

/// 0.3.2 reserved design_description on voice candidates for designed sources, while prompt-guided
/// cloning publishes its style guidance there on reference candidates too; the audition run of such
/// a clone failed the frozen CHECK at publication. Upgrading rebuilds the table around the relaxed
/// rule, keeps every stored candidate, and admits the prompt-guided shape.
#[tokio::test]
async fn upgrading_0_3_2_admits_prompt_guided_reference_voice_candidates() {
    let pool = shipped(4).await;
    for run in ["clone", "guided"] {
        sqlx::query(
            "INSERT INTO spaces_runs(id,scope_kind,home_space_id,principal_kind,principal_id,kind, \
             state,policy_snapshot,input_snapshot,budget,created_at,updated_at,completed_at) \
             VALUES($1,'application',NULL,'user',$1,'workflow','succeeded','{}', \
             '{\"workflow\":\"speech_voice_candidate_v1\"}','{}',1,1,1)",
        )
        .bind(run)
        .execute(&pool)
        .await
        .expect("run");
    }
    sqlx::query(
        "INSERT INTO spaces_voice_candidates( \
         id,run_id,source_kind,language,sample_text,reference_text,input_source_revision,input_byte_length, \
         reference_asset_path,reference_sha256,reference_byte_length,preview_asset_path,preview_sha256, \
         preview_sample_rate,preview_sample_count,model_spec_id,seed,synthesis_config,authorization, \
         status,created_at) \
         VALUES('clone','clone','reference','en','Sample',NULL,$1,32044,'/clone.wav',$2,32044, \
         '/clone-preview.wav',$3,48000,48000,'voxcpm2-2b-gguf-q8_0-v1',0,$4,$5,'ready',1)",
    )
    .bind(format!("sha256:{}", "a".repeat(64)))
    .bind("b".repeat(64))
    .bind("c".repeat(64))
    .bind(r#"{"speed":1.0,"sampleRate":48000,"seed":0}"#)
    .bind(r#"{"referenceTranscriptConfirmed":false}"#)
    .execute(&pool)
    .await
    .expect("stored candidate");

    super::super::migrate(&pool).await.expect("upgrade");

    assert_eq!(schema(&pool).await, schema(&super::migrated().await).await);
    let carried: (i64, Option<String>) = sqlx::query_as(
        "SELECT COUNT(*), MAX(design_description) FROM spaces_voice_candidates WHERE id='clone'",
    )
    .fetch_one(&pool)
    .await
    .expect("carried candidate");
    assert_eq!(
        carried,
        (1, None),
        "the rebuilt table keeps stored candidates"
    );
    sqlx::query(
        "INSERT INTO spaces_voice_candidates( \
         id,run_id,source_kind,language,sample_text,design_description,reference_text, \
         input_source_revision,input_byte_length,reference_asset_path,reference_sha256,reference_byte_length, \
         preview_asset_path,preview_sha256,preview_sample_rate,preview_sample_count,model_spec_id,seed, \
         synthesis_config,authorization,status,created_at) \
         VALUES('guided','guided','reference','en','Sample','沉稳磁性的男声',NULL,$1,32044, \
         '/guided.wav',$2,32044,'/guided-preview.wav',$3,48000,48000,'voxcpm2-2b-gguf-q8_0-v1',0,$4,$5, \
         'ready',1)",
    )
    .bind(format!("sha256:{}", "d".repeat(64)))
    .bind("e".repeat(64))
    .bind("f".repeat(64))
    .bind(r#"{"speed":1.0,"sampleRate":48000,"seed":0}"#)
    .bind(r#"{"declared":true,"referenceTranscriptConfirmed":false}"#)
    .execute(&pool)
    .await
    .expect("prompt guidance is a reference candidate field");
}

/// 0.3.x accepted ten minutes of media for source separation, the most Spleeter held in memory.
/// HTDemucs streams its source, so upgrading rebuilds the table around a six-hour bound and keeps
/// every stored separation with the (since retired) model that made it.
#[tokio::test]
async fn upgrading_0_3_x_admits_six_hour_source_separations() {
    let pool = shipped(5).await;
    for statement in [
        "INSERT INTO spaces(id,kind,title,created_at,updated_at) VALUES('s','space','S',1,1)",
        "INSERT INTO spaces_nodes(id,space_id,kind,initial_x,initial_y,data,created_at,updated_at) \
         VALUES('talk','s','video',0,0,'{\"kind\":\"video\",\"title\":\"Talk\"}',1,1)",
    ] {
        sqlx::query(statement).execute(&pool).await.expect("fixture");
    }
    for run in ["spleeter", "htdemucs"] {
        sqlx::query(
            "INSERT INTO spaces_runs(id,scope_kind,home_space_id,principal_kind,principal_id,kind, \
             state,policy_snapshot,input_snapshot,budget,created_at,updated_at,completed_at) \
             VALUES($1,'space','s','user',$1,'workflow','succeeded','{}', \
             '{\"workflow\":\"speech_source_separation_v1\"}','{}',1,1,1)",
        )
        .bind(run)
        .execute(&pool)
        .await
        .expect("run");
    }
    let separate = |id: &'static str, model: &'static str, duration_ms: i64| {
        let pool = pool.clone();
        async move {
            let samples = duration_ms * 441 / 10;
            sqlx::query(
                "INSERT INTO spaces_speech_source_separations( \
                 id,space_id,run_id,source_node_id,source_asset_path,source_revision, \
                 source_duration_ms,model_spec_id,background_asset_path,vocals_asset_path, \
                 vocals_sha256,background_sha256,background_sample_rate,background_sample_count, \
                 background_duration_ms,created_at) \
                 VALUES($1,'s',$1,'talk','/talk.mp4',$2,$3,$4,$5,$6,$7,$8,44100,$9,$3,1)",
            )
            .bind(id)
            .bind(format!("sha256:{}", "a".repeat(64)))
            .bind(duration_ms)
            .bind(model)
            .bind(format!("/{id}-background.wav"))
            .bind(format!("/{id}-vocals.wav"))
            .bind("b".repeat(64))
            .bind("c".repeat(64))
            .bind(samples)
            .execute(&pool)
            .await
        }
    };
    separate("spleeter", "spleeter-2stems-fp16-cpu-v1", 600_000)
        .await
        .expect("stored separation");

    super::super::migrate(&pool).await.expect("upgrade");

    assert_eq!(schema(&pool).await, schema(&super::migrated().await).await);
    let carried: Vec<(String, String, i64)> = sqlx::query_as(
        "SELECT id,model_spec_id,source_duration_ms FROM spaces_speech_source_separations",
    )
    .fetch_all(&pool)
    .await
    .expect("carried separation");
    assert_eq!(
        carried,
        [(
            "spleeter".to_owned(),
            "spleeter-2stems-fp16-cpu-v1".to_owned(),
            600_000
        )]
    );
    separate("htdemucs", "htdemucs-q8_0-v1", 21_600_000)
        .await
        .expect("a six-hour separation is admitted");
}

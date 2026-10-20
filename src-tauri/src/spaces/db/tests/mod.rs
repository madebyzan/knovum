use sqlx::{sqlite::SqlitePoolOptions, Row, SqlitePool};

async fn migrated() -> SqlitePool {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("database");
    super::migrate(&pool).await.expect("baseline");
    pool
}

/// Every table the schema owns. Virtual tables and the shadow tables FTS5 creates for them are
/// storage of the engine's own making, so they carry neither keys nor indexes of ours.
async fn table_names(pool: &SqlitePool) -> Vec<String> {
    let virtual_tables: Vec<String> = sqlx::query_scalar(
        "SELECT name FROM sqlite_master WHERE type='table' AND sql LIKE 'CREATE VIRTUAL TABLE%'",
    )
    .fetch_all(pool)
    .await
    .expect("virtual tables");
    let tables: Vec<String> = sqlx::query_scalar(
        "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite\\_%' \
         ESCAPE '\\' AND name NOT LIKE '\\_%' ESCAPE '\\' ORDER BY name",
    )
    .fetch_all(pool)
    .await
    .expect("tables");
    tables
        .into_iter()
        .filter(|table| {
            !virtual_tables
                .iter()
                .any(|owner| table == owner || table.starts_with(&format!("{owner}_")))
        })
        .collect()
}

/// A foreign key without an index on its child columns turns every parent delete into a scan of
/// the whole child table, so removing one Space or one node slows down with the size of the
/// library. The rule holds for every foreign key, with no table-by-table exceptions to argue
/// about: a partial index cannot serve a foreign key, and the index must seek the child rows. A
/// unique index over a prefix of the child columns does seek them — it identifies a single row,
/// which the engine then checks for the remaining columns — so it counts, and demanding a longer
/// index on top of it would only duplicate the table key on every write.
#[tokio::test]
async fn every_foreign_key_is_indexed() {
    let pool = migrated().await;
    let mut unindexed = Vec::new();
    for table in table_names(&pool).await {
        let keys = sqlx::query("SELECT * FROM pragma_foreign_key_list($1)")
            .bind(&table)
            .fetch_all(&pool)
            .await
            .expect("foreign keys");
        if keys.is_empty() {
            continue;
        }
        let mut indexes = Vec::new();
        for index in sqlx::query("SELECT * FROM pragma_index_list($1)")
            .bind(&table)
            .fetch_all(&pool)
            .await
            .expect("indexes")
        {
            let name: String = index.get("name");
            let partial: i64 = index.get("partial");
            let unique: i64 = index.get("unique");
            if partial == 1 {
                continue;
            }
            let columns: Vec<String> =
                sqlx::query_scalar("SELECT name FROM pragma_index_info($1) ORDER BY seqno")
                    .bind(&name)
                    .fetch_all(&pool)
                    .await
                    .expect("index columns");
            indexes.push((columns, unique == 1));
        }
        let mut columns_by_foreign_key: std::collections::BTreeMap<i64, Vec<(i64, String)>> =
            std::collections::BTreeMap::new();
        for key in &keys {
            let id: i64 = key.get("id");
            let seq: i64 = key.get("seq");
            let from: String = key.get("from");
            columns_by_foreign_key
                .entry(id)
                .or_default()
                .push((seq, from));
        }
        for (_, mut columns) in columns_by_foreign_key {
            columns.sort_by_key(|(seq, _)| *seq);
            let child: Vec<String> = columns.into_iter().map(|(_, name)| name).collect();
            let covered = indexes.iter().any(|(index, unique)| {
                if index.len() >= child.len() {
                    index[..child.len()] == child[..]
                } else {
                    *unique && child[..index.len()] == index[..]
                }
            });
            if !covered {
                unindexed.push(format!("{table}({})", child.join(",")));
            }
        }
    }
    assert_eq!(unindexed, Vec::<String>::new());
}

/// A full-text index has exactly one key, its rowid: FTS5 supports no secondary index, so a
/// statement that addresses a row by the owner id stored beside the text reads the entire index.
/// Per-row triggers turn that into quadratic work — deleting one Space's knowledge took seconds —
/// so every trigger that maintains an index must address its rows by rowid.
#[tokio::test]
async fn search_index_rows_are_addressed_by_row() {
    let pool = migrated().await;
    let virtual_tables: Vec<String> = sqlx::query_scalar(
        "SELECT name FROM sqlite_master WHERE type='table' AND sql LIKE 'CREATE VIRTUAL TABLE%'",
    )
    .fetch_all(&pool)
    .await
    .expect("virtual tables");
    let triggers: Vec<String> =
        sqlx::query_scalar("SELECT sql FROM sqlite_master WHERE type='trigger'")
            .fetch_all(&pool)
            .await
            .expect("triggers");
    let mut scanning = Vec::new();
    for sql in triggers {
        for table in &virtual_tables {
            for statement in sql.split(';') {
                let Some((_, predicate)) =
                    statement.split_once(&format!("DELETE FROM {table} WHERE "))
                else {
                    continue;
                };
                if !predicate.trim_start().starts_with("rowid") {
                    scanning.push(format!("{table}: {}", predicate.trim()));
                }
            }
        }
    }
    assert_eq!(scanning, Vec::<String>::new());
}

/// What the rowid addressing buys, stated as behaviour: the index follows its owner exactly, so a
/// deleted Space leaves nothing searchable behind and a retitled container is found under its new
/// title and not its old one.
#[tokio::test]
async fn deleting_and_retitling_keeps_the_search_index_exact() {
    let pool = migrated().await;
    for space in ["kept", "removed"] {
        sqlx::query(
            "INSERT INTO spaces(id,kind,title,created_at,updated_at) VALUES($1,'space','S',1,1)",
        )
        .bind(space)
        .execute(&pool)
        .await
        .expect("space");
        for index in 0..20 {
            sqlx::query("INSERT INTO spaces_structure_nodes(id,space_id,kind,node_id,parent_id,order_index,title,locked) VALUES($1,$2,'section',NULL,NULL,$3,$4,0)")
                .bind(format!("{space}-{index}")).bind(space).bind(index)
                .bind(format!("Chapter {index}")).execute(&pool).await.expect("container");
        }
    }
    let indexed: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM spaces_structure_title_fts")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(indexed, 40);

    sqlx::query("UPDATE spaces_structure_nodes SET title='Appendix' WHERE id='kept-3'")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("DELETE FROM spaces WHERE id='removed'")
        .execute(&pool)
        .await
        .unwrap();

    let remaining: Vec<String> = sqlx::query_scalar(
        "SELECT structure_node_id FROM spaces_structure_title_fts \
         WHERE spaces_structure_title_fts MATCH 'Appendix OR Chapter' \
         ORDER BY structure_node_id",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(remaining.len(), 20, "only the kept Space stays searchable");
    assert!(remaining.iter().all(|id| id.starts_with("kept-")));
    let appendix: Vec<String> = sqlx::query_scalar(
        "SELECT structure_node_id FROM spaces_structure_title_fts \
         WHERE spaces_structure_title_fts MATCH 'Appendix'",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(appendix, ["kept-3"]);
}

/// A table whose rows must outlive the rows they name. Presence in a retained-reference table
/// means "keep this file", so it has to cascade with its owner or the file is retained forever;
/// presence in a tombstone table means "delete this thing", so it has to survive its owner or the
/// deletion never happens. Only the second kind may hold `space_id` without a key, and each one
/// is listed here with that reason.
const TOMBSTONE_TABLES: [&str; 1] = ["spaces_retrieval_invalidations"];

/// Every table that holds Space-scoped rows must let the Space own them, otherwise deleting a
/// Space leaves rows that no feature can reach and no reclamation pass can drain. Ownership may
/// be transitive: a composite key that carries `space_id` into another Space-owned table counts.
#[tokio::test]
async fn space_scoped_rows_belong_to_their_space() {
    let pool = migrated().await;
    let mut unowned = Vec::new();
    for table in table_names(&pool).await {
        if table == "spaces" || TOMBSTONE_TABLES.contains(&table.as_str()) {
            continue;
        }
        let columns: Vec<String> = sqlx::query_scalar("SELECT name FROM pragma_table_info($1)")
            .bind(&table)
            .fetch_all(&pool)
            .await
            .expect("columns");
        if !columns.iter().any(|column| column == "space_id") {
            continue;
        }
        let owned = sqlx::query("SELECT * FROM pragma_foreign_key_list($1)")
            .bind(&table)
            .fetch_all(&pool)
            .await
            .expect("foreign keys")
            .into_iter()
            .any(|key| {
                let from: String = key.get("from");
                from == "space_id"
            });
        if !owned {
            unowned.push(table);
        }
    }
    assert_eq!(unowned, Vec::<String>::new());
}

/// Running the migrator against an already migrated database must be a no-op, which is what
/// makes an additive migration safe to ship: the installed application applies what is missing
/// and leaves everything else alone.
#[tokio::test]
async fn migrating_an_up_to_date_database_changes_nothing() {
    let pool = migrated().await;
    let before: Vec<String> =
        sqlx::query_scalar("SELECT sql FROM sqlite_master WHERE sql IS NOT NULL ORDER BY name")
            .fetch_all(&pool)
            .await
            .expect("schema");
    sqlx::query(
        "INSERT INTO spaces(id,kind,title,created_at,updated_at) VALUES('keep','space','Keep',1,1)",
    )
    .execute(&pool)
    .await
    .expect("user row");

    super::migrate(&pool).await.expect("second run");

    let after: Vec<String> =
        sqlx::query_scalar("SELECT sql FROM sqlite_master WHERE sql IS NOT NULL ORDER BY name")
            .fetch_all(&pool)
            .await
            .expect("schema");
    assert_eq!(before, after);
    let kept: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM spaces WHERE id='keep'")
        .fetch_one(&pool)
        .await
        .expect("user row preserved");
    assert_eq!(kept, 1);
}

#[tokio::test]
async fn image_object_removal_is_a_valid_derivation_action() {
    let pool = migrated().await;
    sqlx::query("INSERT INTO spaces(id,kind,title,created_at,updated_at) VALUES('space','space','Space',1,1)")
        .execute(&pool).await.expect("space");
    sqlx::query("INSERT INTO spaces_nodes(id,space_id,kind,initial_x,initial_y,data,created_at,updated_at) VALUES('output','space','image',0,0,'{\"kind\":\"image\",\"title\":\"Output\"}',1,1)")
        .execute(&pool).await.expect("image output");
    sqlx::query("INSERT INTO spaces_knowledge_derivations(id,space_id,output_node_id,action_kind,action_version,engine,status,created_at,updated_at) VALUES('job','space','output','remove-image-object',1,'dashscope:wanx2.1-imageedit:mask','running',1,1)")
        .execute(&pool).await.expect("object-removal derivation");
}

#[tokio::test]
async fn obsolete_migration_history_is_rejected_without_losing_user_rows() {
    let pool = migrated().await;
    sqlx::query(
        "INSERT INTO spaces(id,kind,title,created_at,updated_at) VALUES('keep','space','Keep',1,1)",
    )
    .execute(&pool)
    .await
    .expect("user row");
    sqlx::query("INSERT INTO _sqlx_migrations(version,description,success,checksum,execution_time) VALUES(99,'from a newer release',1,x'00',0)").execute(&pool).await.expect("newer version");
    let error = super::migrate(&pool)
        .await
        .expect_err("a database from a newer release must fail");
    assert!(matches!(
        error,
        sqlx::migrate::MigrateError::VersionMissing(99)
    ));
    let title: String = sqlx::query_scalar("SELECT title FROM spaces WHERE id='keep'")
        .fetch_one(&pool)
        .await
        .expect("original data preserved");
    assert_eq!(title, "Keep");
}

#[tokio::test]
async fn an_edited_applied_migration_is_rejected_without_resetting_the_database() {
    let pool = migrated().await;
    sqlx::query(
        "INSERT INTO spaces(id,kind,title,created_at,updated_at) VALUES('keep','space','Keep',1,1)",
    )
    .execute(&pool)
    .await
    .expect("user row");
    sqlx::query("UPDATE _sqlx_migrations SET checksum=x'00' WHERE version=1")
        .execute(&pool)
        .await
        .expect("edited baseline");
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
    let pool = migrated().await;
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
    let pool = migrated().await;
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
    let pool = migrated().await;
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

    // A model identifier is catalogue data, so the schema stores whichever engine produced the
    // candidate and the speech catalogue decides which engines may be offered.
    sqlx::query("UPDATE spaces_voice_candidates SET model_spec_id='voxcpm2-2b-gguf-q8_0-v1' WHERE id='moss-reference'")
        .execute(&pool).await.expect("re-engineered candidate");

    sqlx::query(
        "INSERT INTO spaces_voice_candidates( \
         id,run_id,source_kind,language,sample_text,design_description,reference_text, \
         reference_asset_path,reference_sha256,reference_byte_length,preview_asset_path,preview_sha256, \
         preview_sample_rate,preview_sample_count,model_spec_id,seed,synthesis_config,authorization,status,created_at) \
         VALUES('moss-designed','moss-designed','designed','en','Sample','Warm','Sample', \
         '/moss-designed.wav',?,32044,'/moss-designed-preview.wav',?,48000,48000, \
         'voxcpm2-2b-gguf-orig-v1',0,?,?,'ready',1)",
    )
    .bind("d".repeat(64))
    .bind("e".repeat(64))
    .bind(r#"{"speed":1.0,"sampleRate":48000,"seed":0}"#)
    .bind(r#"{"declared":true,"referenceTranscriptConfirmed":true}"#)
    .execute(&pool)
    .await
    .expect("designed candidate");

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

    // The stored sample rate drives the duration invariant, so a 48 kHz engine is accepted on
    // its own terms instead of being measured against a rate written into the constraint.
    sqlx::query(
        "INSERT INTO spaces_speech_source_separations( \
         id,space_id,run_id,source_node_id,source_asset_path,source_revision,source_duration_ms,model_spec_id, \
         background_asset_path,vocals_asset_path,vocals_sha256,background_sha256,background_sample_rate, \
         background_sample_count,background_duration_ms,created_at) \
         VALUES('moss-designed','space','moss-designed','source','/source-48k.wav',?,1000, \
         'spleeter-2stems-fp32-cpu-v1','/background-48k.wav','/vocals-48k.wav',?,?,48000,48000,1000, \
         1)",
    )
    .bind(format!("sha256:{}", "7".repeat(64)))
    .bind("3".repeat(64))
    .bind("4".repeat(64))
    .execute(&pool)
    .await
    .expect("48 kHz separation");
}

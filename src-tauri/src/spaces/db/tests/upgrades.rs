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

/// Every PDF picture was rendered by a renderer that drew them upside down on macOS. Upgrading
/// withdraws each PDF coverage record and queues the document to be parsed again, once; Office
/// documents, whose pictures were never rendered, keep theirs.
#[tokio::test]
async fn upgrading_rerenders_every_pdf_picture_once() {
    let pool = shipped(4).await;
    sqlx::query(
        "INSERT INTO spaces(id,kind,title,created_at,updated_at) VALUES('s','space','S',1,1)",
    )
    .execute(&pool)
    .await
    .expect("space");
    let coverage = |parser: &str| serde_json::json!({ "parser": parser, "resources": [] });
    for (id, deleted_at, data) in [
        (
            "pdf",
            None,
            serde_json::json!({"kind":"file","title":"Brief","sourceRevision":"r1","documentCoverage":coverage("pdf-inspector")}),
        ),
        (
            "queued",
            None,
            serde_json::json!({"kind":"file","title":"Brief","sourceRevision":"r1","documentCoverage":coverage("pdf-inspector")}),
        ),
        (
            "trashed",
            Some(5_i64),
            serde_json::json!({"kind":"file","title":"Brief","sourceRevision":"r1","documentCoverage":coverage("pdf-inspector")}),
        ),
        (
            "deck",
            None,
            serde_json::json!({"kind":"file","title":"Brief","sourceRevision":"r1","documentCoverage":coverage("office-oxide")}),
        ),
    ] {
        sqlx::query(
            "INSERT INTO spaces_nodes(id,space_id,kind,initial_x,initial_y,data,created_at,updated_at,deleted_at) \
             VALUES($1,'s','file',0,0,$2,1,1,$3)",
        )
        .bind(id)
        .bind(data.to_string())
        .bind(deleted_at)
        .execute(&pool)
        .await
        .expect("node");
    }
    sqlx::query(
        "INSERT INTO spaces_enrichment_jobs(id,space_id,node_id,job_type,status,source_revision,attempts,created_at,updated_at) \
         VALUES('waiting','s','queued','parse_document','pending','r1',0,1,1)",
    )
    .execute(&pool)
    .await
    .expect("job");

    super::super::migrate(&pool).await.expect("upgrade");

    assert_eq!(schema(&pool).await, schema(&super::migrated().await).await);
    let covered: Vec<(String, bool)> = sqlx::query_as(
        "SELECT id, json_extract(data,'$.documentCoverage') IS NOT NULL FROM spaces_nodes ORDER BY id",
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
    let queued: Vec<(String, String, String)> = sqlx::query_as(
        "SELECT node_id,status,source_revision FROM spaces_enrichment_jobs \
         WHERE job_type='parse_document' ORDER BY node_id",
    )
    .fetch_all(&pool)
    .await
    .expect("jobs");
    assert_eq!(
        queued,
        [
            ("pdf".to_owned(), "pending".to_owned(), "r1".to_owned()),
            ("queued".to_owned(), "pending".to_owned(), "r1".to_owned()),
        ]
    );
}

/// The first text recognition kept each node's text in its record and read pictures nobody can
/// tell apart afterwards. Upgrading withdraws every record and the jobs that answered them, so each
/// node is read again per picture, and leaves the passages for the new reading to replace.
#[tokio::test]
async fn upgrading_reads_every_picture_again() {
    let pool = shipped(5).await;
    let record = serde_json::json!({"sourceRevision":"r1","modelSpecId":"ocr","generatedAt":1,"text":"Revenue"});
    sqlx::query(
        "INSERT INTO spaces(id,kind,title,created_at,updated_at) VALUES('s','space','S',1,1)",
    )
    .execute(&pool)
    .await
    .expect("space");
    for (id, kind, data) in [
        (
            "brief",
            "file",
            serde_json::json!({"kind":"file","title":"Brief","sourceRevision":"r1","textRecognition":record}),
        ),
        (
            "photo",
            "image",
            serde_json::json!({"kind":"image","title":"Photo","sourceRevision":"r1","textRecognition":record}),
        ),
        (
            "note",
            "note",
            serde_json::json!({"kind":"note","title":"Note"}),
        ),
    ] {
        sqlx::query(
            "INSERT INTO spaces_nodes(id,space_id,kind,initial_x,initial_y,data,created_at,updated_at) \
             VALUES($1,'s',$2,0,0,$3,1,1)",
        )
        .bind(id)
        .bind(kind)
        .bind(data.to_string())
        .execute(&pool)
        .await
        .expect("node");
    }
    for statement in [
        "INSERT INTO spaces_enrichment_jobs(id,space_id,node_id,job_type,status,source_revision,attempts,created_at,updated_at) \
         VALUES('read','s','brief','recognize_text','succeeded','r1',1,1,1)",
        "INSERT INTO spaces_enrichment_jobs(id,space_id,node_id,job_type,status,source_revision,attempts,created_at,updated_at) \
         VALUES('described','s','photo','analyze_visual','succeeded','v1',1,1,1)",
    ] {
        sqlx::query(statement).execute(&pool).await.expect("job");
    }

    super::super::migrate(&pool).await.expect("upgrade");

    assert_eq!(schema(&pool).await, schema(&super::migrated().await).await);
    let nodes: Vec<(String, bool, String)> = sqlx::query_as(
        "SELECT id, json_extract(data,'$.textRecognition') IS NULL, json_extract(data,'$.title') \
         FROM spaces_nodes ORDER BY id",
    )
    .fetch_all(&pool)
    .await
    .expect("nodes");
    assert_eq!(
        nodes,
        [
            ("brief".to_owned(), true, "Brief".to_owned()),
            ("note".to_owned(), true, "Note".to_owned()),
            ("photo".to_owned(), true, "Photo".to_owned()),
        ]
    );
    let jobs: Vec<String> = sqlx::query_scalar("SELECT id FROM spaces_enrichment_jobs ORDER BY id")
        .fetch_all(&pool)
        .await
        .expect("jobs");
    assert_eq!(jobs, ["described"]);
}

async fn insert_node(
    pool: &SqlitePool,
    id: &str,
    data: serde_json::Value,
    deleted_at: Option<i64>,
) {
    sqlx::query(
        "INSERT INTO spaces_nodes(id,space_id,kind,initial_x,initial_y,data,created_at,updated_at,deleted_at) \
         VALUES($1,'s',$2,0,0,$3,1,1,$4)",
    )
    .bind(id)
    .bind(data["kind"].as_str().expect("kind"))
    .bind(data.to_string())
    .bind(deleted_at)
    .execute(pool)
    .await
    .expect("node");
}

async fn insert_job(pool: &SqlitePool, job: (&str, &str, &str, &str, &str, Option<&str>, i64)) {
    let (id, node, kind, status, revision, error, updated_at) = job;
    sqlx::query(
        "INSERT INTO spaces_enrichment_jobs(id,space_id,node_id,job_type,status,source_revision, \
         attempts,error,created_at,updated_at) VALUES($1,'s',$2,$3,$4,$5,1,$6,1,$7)",
    )
    .bind(id)
    .bind(node)
    .bind(kind)
    .bind(status)
    .bind(revision)
    .bind(error)
    .bind(updated_at)
    .execute(pool)
    .await
    .expect("job");
}

async fn space(version: i64) -> SqlitePool {
    let pool = shipped(version).await;
    sqlx::query(
        "INSERT INTO spaces(id,kind,title,created_at,updated_at) VALUES('s','space','S',1,1)",
    )
    .execute(&pool)
    .await
    .expect("space");
    pool
}

const REFUSED: &str = "textRecognition field 'text' is not supported";

/// A build that carried 0005 without 0006 refused every result written to a node holding the first
/// reader's record, so the re-parses 0005 queued failed. Upgrading revives the latest refused job of
/// each live node once and leaves every other outcome as it was.
#[tokio::test]
async fn upgrading_retries_the_work_the_retired_record_refused() {
    let pool = space(6).await;
    let file = serde_json::json!({"kind":"file","title":"Brief","sourceRevision":"r1"});
    for (id, deleted_at) in [
        ("brief", None),
        ("busy", None),
        ("photo", None),
        ("trashed", Some(5)),
    ] {
        insert_node(&pool, id, file.clone(), deleted_at).await;
    }
    for job in [
        (
            "first",
            "brief",
            "parse_document",
            "failed",
            "r1",
            Some(REFUSED),
            2,
        ),
        (
            "again",
            "brief",
            "parse_document",
            "failed",
            "r1",
            Some(REFUSED),
            3,
        ),
        (
            "other",
            "photo",
            "parse_document",
            "failed",
            "r1",
            Some("PDF requires a password"),
            2,
        ),
        (
            "bin",
            "trashed",
            "parse_document",
            "failed",
            "r1",
            Some(REFUSED),
            2,
        ),
        (
            "stale",
            "busy",
            "parse_document",
            "failed",
            "r1",
            Some(REFUSED),
            2,
        ),
        ("live", "busy", "parse_document", "pending", "r1", None, 4),
    ] {
        insert_job(&pool, job).await;
    }

    super::super::migrate(&pool).await.expect("upgrade");

    assert_eq!(schema(&pool).await, schema(&super::migrated().await).await);
    let jobs: Vec<(String, String, Option<String>)> =
        sqlx::query_as("SELECT id,status,error FROM spaces_enrichment_jobs ORDER BY id")
            .fetch_all(&pool)
            .await
            .expect("jobs");
    let refused = Some(REFUSED.to_owned());
    assert_eq!(
        jobs,
        [
            ("again".to_owned(), "pending".to_owned(), None),
            ("bin".to_owned(), "failed".to_owned(), refused.clone()),
            ("first".to_owned(), "failed".to_owned(), refused.clone()),
            ("live".to_owned(), "pending".to_owned(), None),
            (
                "other".to_owned(),
                "failed".to_owned(),
                Some("PDF requires a password".to_owned())
            ),
            ("stale".to_owned(), "failed".to_owned(), refused),
        ]
    );
}

/// A node as the app saved it, carrying the knowledge revision every write derives.
fn written(mut data: serde_json::Value) -> serde_json::Value {
    crate::spaces::knowledge::model::revision::apply(&mut data).expect("revision");
    data
}

/// A library exactly as the first text recognition left it opens with every node valid for the
/// current release, its PDFs queued to be parsed again and its pictures owed a new reading.
#[tokio::test]
async fn a_library_the_first_text_recognition_wrote_opens_and_is_read_again() {
    let pool = space(4).await;
    let record = serde_json::json!({"sourceRevision":"r1","modelSpecId":"ocr","generatedAt":1,"text":"Revenue"});
    let coverage = serde_json::json!({
        "schemaVersion":1,"parser":"pdf-inspector","sourceRevision":"r1","pageCount":1,
        "pagesRequiringRecognition":[],"pagesWithExtractedText":[1],"pagesWithoutExtractableContent":[],
        "resources":[],"occurrences":[],"warnings":[]
    });
    insert_node(
        &pool,
        "brief",
        written(serde_json::json!({"kind":"file","title":"Brief","assetPath":"/a/brief.pdf","fileName":"brief.pdf",
            "sourceRevision":"r1","documentCoverage":coverage,"textRecognition":record})),
        None,
    )
    .await;
    insert_node(
        &pool,
        "photo",
        written(serde_json::json!({"kind":"image","title":"Photo","assetPath":"/a/photo.png","fileName":"photo.png",
            "sourceRevision":"r1","textRecognition":record})),
        None,
    )
    .await;
    for job in [
        (
            "parsed",
            "brief",
            "parse_document",
            "succeeded",
            "r1",
            None,
            1,
        ),
        (
            "read-brief",
            "brief",
            "recognize_text",
            "succeeded",
            "r1",
            None,
            1,
        ),
        (
            "read-photo",
            "photo",
            "recognize_text",
            "succeeded",
            "r1",
            None,
            1,
        ),
    ] {
        insert_job(&pool, job).await;
    }

    super::super::migrate(&pool).await.expect("upgrade");

    let nodes: Vec<(String, String, String)> =
        sqlx::query_as("SELECT id,kind,data FROM spaces_nodes ORDER BY id")
            .fetch_all(&pool)
            .await
            .expect("nodes");
    let mut tx = super::super::begin_write(&pool).await.expect("tx");
    for (id, kind, data) in &nodes {
        let data: serde_json::Value = serde_json::from_str(data).expect("json");
        crate::spaces::knowledge::validation::validate_node(&mut tx, "s", kind, &data)
            .await
            .unwrap_or_else(|error| panic!("{id} is invalid after upgrading: {error}"));
    }
    drop(tx);
    let jobs: Vec<(String, String)> = sqlx::query_as(
        "SELECT node_id,job_type FROM spaces_enrichment_jobs WHERE status='pending' ORDER BY node_id",
    )
    .fetch_all(&pool)
    .await
    .expect("jobs");
    assert_eq!(jobs, [("brief".to_owned(), "parse_document".to_owned())]);
    crate::spaces::assets::enrichment::jobs::intent::enqueue_missing_recognition(&pool, 2)
        .await
        .expect("sweep");
    let owed: Vec<String> = sqlx::query_scalar(
        "SELECT node_id FROM spaces_enrichment_jobs WHERE job_type='recognize_text' AND status='pending'",
    )
    .fetch_all(&pool)
    .await
    .expect("owed");
    assert_eq!(
        owed,
        ["photo"],
        "the document is read once it is parsed again"
    );
}

use super::{open, record_batch, Command, RunEventsOnly, Stream, Streamed, MAX_UPDATE_TEXT_BYTES};
use crate::external_agents::session::AgentSessionUpdate;

async fn pool_with_run() -> sqlx::SqlitePool {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    crate::spaces::db::migrate(&pool).await.unwrap();
    sqlx::raw_sql(
        "INSERT INTO spaces(id,kind,title,created_at,updated_at) VALUES('space','space','S',1,1);
         INSERT INTO spaces_runs(id,scope_kind,home_space_id,principal_kind,principal_id,kind,
           state,policy_snapshot,input_snapshot,budget,created_at,updated_at)
         VALUES('run','space','space','external_agent','session','external_agent','running',
           '{}','{}','{}',1,1);",
    )
    .execute(&pool)
    .await
    .unwrap();
    pool
}

fn chunks(count: usize) -> Vec<Streamed> {
    (0..count)
        .map(|index| Streamed::of(AgentSessionUpdate::Message(format!("chunk-{index} "))).unwrap())
        .collect()
}

#[tokio::test]
async fn a_batch_of_chunks_costs_one_transaction() {
    let pool = pool_with_run().await;
    record_batch(&pool, "run", &RunEventsOnly, &chunks(400))
        .await
        .unwrap();
    let (recorded, commits): (i64, i64) = sqlx::query_as(
        "SELECT COUNT(*),COUNT(DISTINCT created_at) FROM spaces_run_events WHERE run_id='run'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!((recorded, commits), (400, 1));
}

#[tokio::test]
async fn a_recorded_stream_reads_back_in_arrival_order() {
    let pool = pool_with_run().await;
    let stream = open(pool.clone(), "run".to_owned(), RunEventsOnly);
    for index in 0..100 {
        stream
            .record(AgentSessionUpdate::Message(format!("chunk-{index} ")))
            .unwrap();
    }
    stream.flush().await.unwrap();
    let answer: String = sqlx::query_scalar(
        "SELECT group_concat(json_extract(payload,'$.text'),'') FROM
         (SELECT payload FROM spaces_run_events WHERE run_id='run' ORDER BY sequence)",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        answer,
        (0..100)
            .map(|index| format!("chunk-{index} "))
            .collect::<String>()
    );
}

#[tokio::test]
async fn a_flush_reports_the_failure_the_recorder_met() {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    crate::spaces::db::migrate(&pool).await.unwrap();
    sqlx::raw_sql("PRAGMA foreign_keys=ON;")
        .execute(&pool)
        .await
        .unwrap();
    let stream = open(pool, "absent-run".to_owned(), RunEventsOnly);
    stream
        .record(AgentSessionUpdate::Message("orphan".into()))
        .unwrap();
    assert!(stream.flush().await.is_err());
    assert!(stream.flush().await.is_ok());
}

#[test]
fn a_thought_leaves_no_record() {
    assert!(Streamed::of(AgentSessionUpdate::Thought("private".into())).is_none());
}

#[test]
fn the_queue_rejects_a_burst_above_its_capacity() {
    let (updates, _recorded) = tokio::sync::mpsc::channel::<Command>(1);
    let stream = Stream { updates };
    assert!(stream
        .record(AgentSessionUpdate::Message("first".into()))
        .is_ok());
    assert!(stream
        .record(AgentSessionUpdate::Message("second".into()))
        .is_err());
    assert!(stream
        .record(AgentSessionUpdate::Thought("ignored".into()))
        .is_ok());
}

#[test]
fn the_queue_rejects_an_oversized_update() {
    let (updates, _recorded) = tokio::sync::mpsc::channel::<Command>(1);
    let stream = Stream { updates };
    assert!(stream
        .record(AgentSessionUpdate::Message(
            "x".repeat(MAX_UPDATE_TEXT_BYTES + 1)
        ))
        .is_err());
}

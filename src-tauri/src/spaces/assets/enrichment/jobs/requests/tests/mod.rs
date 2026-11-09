use sqlx::sqlite::SqlitePoolOptions;

use super::ensure_no_active_transcript_consumers;

#[tokio::test]
async fn active_alignment_or_diarization_blocks_retranscription_until_finished() {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    crate::spaces::db::migrate(&pool).await.unwrap();
    sqlx::query(
        "INSERT INTO spaces(id,kind,title,created_at,updated_at) VALUES('space','space','Space',1,1)",
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO spaces_nodes(id,space_id,kind,initial_x,initial_y,data,created_at,updated_at) VALUES('node','space','video',0,0,'{\"kind\":\"video\",\"title\":\"Clip\"}',1,1)",
    )
    .execute(&pool)
    .await
    .unwrap();

    let mut tx = pool.begin().await.unwrap();
    assert!(
        ensure_no_active_transcript_consumers(&mut tx, "space", "node")
            .await
            .is_ok()
    );
    sqlx::query(
        "INSERT INTO spaces_enrichment_jobs(id,space_id,node_id,job_type,status,source_revision,attempts,created_at,updated_at) \
         VALUES('align-1','space','node','align_media_transcript','running','rev-1',1,1,1)",
    )
    .execute(&mut *tx)
    .await
    .unwrap();
    assert!(
        ensure_no_active_transcript_consumers(&mut tx, "space", "node")
            .await
            .is_err()
    );
    sqlx::query("UPDATE spaces_enrichment_jobs SET status='succeeded' WHERE id='align-1'")
        .execute(&mut *tx)
        .await
        .unwrap();
    assert!(
        ensure_no_active_transcript_consumers(&mut tx, "space", "node")
            .await
            .is_ok()
    );
}

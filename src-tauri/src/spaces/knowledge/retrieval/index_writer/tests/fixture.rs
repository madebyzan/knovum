use std::{path::PathBuf, time::Duration};

use lancedb::Connection;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

use super::super::KnowledgeEmbeddingInput;

pub(in crate::spaces::knowledge::retrieval) struct Fixture {
    pub pool: sqlx::SqlitePool,
    pub index: Connection,
    // Declared last: Windows refuses to remove files whose handles are still open, and fields
    // drop in declaration order, so pool and index close before the directory is removed.
    #[allow(dead_code)] // Never read; its drop after pool and index is the cleanup.
    dir: OwnedDir,
}

struct OwnedDir(PathBuf);

impl Drop for OwnedDir {
    fn drop(&mut self) {
        // Best-effort: on Windows, lancedb's dataset cache and sqlx keep the Lance and SQLite
        // files (including the memory-mapped WAL -shm) open for the whole test process, so the
        // directory often cannot be removed while the process lives. Teardown runs after every
        // assertion, and a leaked temp dir on an ephemeral CI runner is harmless, so a failed
        // removal must not turn passing tests into failures.
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

impl Fixture {
    pub async fn new() -> Self {
        let dir = std::env::temp_dir().join(format!("knovum-index-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).expect("test directory");
        let pool = SqlitePoolOptions::new()
            .max_connections(2)
            .connect_with(
                SqliteConnectOptions::new()
                    .filename(dir.join("spaces.db"))
                    .create_if_missing(true)
                    .foreign_keys(true)
                    .busy_timeout(Duration::from_secs(5)),
            )
            .await
            .expect("sqlite");
        crate::spaces::db::migrate(&pool).await.expect("migrations");
        sqlx::query("INSERT INTO spaces(id,kind,title,created_at,updated_at) VALUES ('space','space','Space',1,1)")
            .execute(&pool).await.expect("space");
        sqlx::query("INSERT INTO spaces_retrieval_generations(id,space_id,embedding_spec_id,status,created_at) \
            VALUES ('generation','space','spec','building',1)")
            .execute(&pool).await.expect("generation");
        sqlx::query("INSERT INTO spaces_nodes(id,space_id,kind,initial_x,initial_y,data,created_at,updated_at) \
            VALUES ('node','space','image',0,0,'{\"kind\":\"image\",\"title\":\"Image\",\"sourceRevision\":\"r1\"}',1,1)")
            .execute(&pool).await.expect("node");
        sqlx::query("INSERT INTO spaces_knowledge_chunks(id,space_id,node_id,origin,kind,source_revision,text,locator,created_at,updated_at) \
            VALUES ('c1','space','node','source','image_description','r1','Description','{}',1,1)")
            .execute(&pool).await.expect("chunk");
        let index = lancedb::connect(dir.join("vectors").to_str().expect("path"))
            .manifest_enabled(false)
            .execute()
            .await
            .expect("lancedb");
        Self {
            pool,
            index,
            dir: OwnedDir(dir),
        }
    }

    pub fn input(&self) -> Vec<KnowledgeEmbeddingInput> {
        vec![super::build_input(vec![0.1, 0.2], 2)]
    }

    pub async fn markers(&self) -> i64 {
        sqlx::query_scalar("SELECT COUNT(*) FROM spaces_chunk_embeddings")
            .fetch_one(&self.pool)
            .await
            .expect("marker count")
    }

    pub async fn rows(&self) -> usize {
        self.index
            .open_table(crate::spaces::knowledge::retrieval::schema::TABLE_KNOWLEDGE_CHUNKS)
            .execute()
            .await
            .expect("table")
            .count_rows(None)
            .await
            .expect("vector count")
    }
}

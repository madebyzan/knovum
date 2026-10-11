use sqlx::{Row, SqlitePool};

use crate::spaces::db;

pub(super) const REBUILD_REQUIRED: &str = "knowledge-semantic-index-rebuild-required";

#[derive(Debug, Clone)]
pub(super) struct IndexGeneration {
    pub id: String,
    pub embedding_spec_id: String,
}

pub(super) async fn active(
    pool: &SqlitePool,
    space_id: &str,
    embedding_spec_id: &str,
) -> Result<IndexGeneration, String> {
    find_active(pool, space_id, embedding_spec_id)
        .await?
        .ok_or_else(|| REBUILD_REQUIRED.to_owned())
}

pub(super) async fn find_active(
    pool: &SqlitePool,
    space_id: &str,
    embedding_spec_id: &str,
) -> Result<Option<IndexGeneration>, String> {
    let row = sqlx::query(
        "SELECT id,embedding_spec_id FROM spaces_retrieval_generations \
         WHERE space_id=$1 AND status='active'",
    )
    .bind(space_id)
    .fetch_optional(pool)
    .await
    .map_err(db::sql_error)?;
    let Some(row) = row else { return Ok(None) };
    let generation = decode(row)?;
    if generation.embedding_spec_id != embedding_spec_id {
        return Ok(None);
    }
    Ok(Some(generation))
}

pub(super) async fn begin_rebuild(
    pool: &SqlitePool,
    space_id: &str,
    embedding_spec_id: &str,
) -> Result<IndexGeneration, String> {
    let mut tx = db::begin_write(pool).await?;
    let existing = sqlx::query(
        "SELECT id,embedding_spec_id FROM spaces_retrieval_generations \
         WHERE space_id=$1 AND status='building'",
    )
    .bind(space_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    if let Some(row) = existing {
        let generation = decode(row)?;
        if generation.embedding_spec_id == embedding_spec_id {
            tx.commit().await.map_err(db::sql_error)?;
            return Ok(generation);
        }
    }
    sqlx::query(
        "UPDATE spaces_retrieval_generations SET status='retired',activated_at=NULL \
         WHERE space_id=$1 AND status IN ('active','building')",
    )
    .bind(space_id)
    .execute(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    let generation = IndexGeneration {
        id: uuid::Uuid::new_v4().to_string(),
        embedding_spec_id: embedding_spec_id.to_owned(),
    };
    let now = crate::foundation::clock::unix_time_millis()?;
    sqlx::query(
        "INSERT INTO spaces_retrieval_generations(\
           id,space_id,embedding_spec_id,status,created_at\
         ) VALUES($1,$2,$3,'building',$4)",
    )
    .bind(&generation.id)
    .bind(space_id)
    .bind(&generation.embedding_spec_id)
    .bind(now)
    .execute(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    tx.commit().await.map_err(db::sql_error)?;
    Ok(generation)
}

pub(super) async fn activate(
    pool: &SqlitePool,
    space_id: &str,
    generation: &IndexGeneration,
) -> Result<(), String> {
    let mut tx = db::begin_write(pool).await?;
    let missing: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM spaces_current_knowledge_chunks c \
         LEFT JOIN spaces_chunk_embeddings e \
           ON e.chunk_id=c.id AND e.generation_id=$2 \
          AND e.embedding_spec_id=$3 AND e.source_revision=c.source_revision \
         WHERE c.space_id=$1 AND trim(c.text)<>'' AND e.chunk_id IS NULL",
    )
    .bind(space_id)
    .bind(&generation.id)
    .bind(&generation.embedding_spec_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    if missing != 0 {
        return Err(format!(
            "Knowledge index generation is incomplete: {missing} chunks are missing"
        ));
    }
    let now = crate::foundation::clock::unix_time_millis()?;
    let changed = sqlx::query(
        "UPDATE spaces_retrieval_generations SET status='active',activated_at=$1 \
         WHERE id=$2 AND space_id=$3 AND embedding_spec_id=$4 AND status='building'",
    )
    .bind(now)
    .bind(&generation.id)
    .bind(space_id)
    .bind(&generation.embedding_spec_id)
    .execute(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    if changed.rows_affected() != 1 {
        return Err("Knowledge index generation is no longer publishable".to_owned());
    }
    tx.commit().await.map_err(db::sql_error)
}

pub(super) async fn require_writable(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    generation: &IndexGeneration,
) -> Result<(), String> {
    let valid: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM spaces_retrieval_generations \
         WHERE id=$1 AND embedding_spec_id=$2 AND status IN ('building','active')",
    )
    .bind(&generation.id)
    .bind(&generation.embedding_spec_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    if valid == 1 {
        Ok(())
    } else {
        Err("Knowledge index generation is no longer writable".to_owned())
    }
}

fn decode(row: sqlx::sqlite::SqliteRow) -> Result<IndexGeneration, String> {
    Ok(IndexGeneration {
        id: row.try_get("id").map_err(db::sql_error)?,
        embedding_spec_id: row.try_get("embedding_spec_id").map_err(db::sql_error)?,
    })
}

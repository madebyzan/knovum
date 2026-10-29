//! Storage of answer verifications, one per assistant message.

use sqlx::{sqlite::SqliteRow, Row, Sqlite, SqliteConnection, Transaction};

use super::contracts::{
    AnswerStatement, AnswerVerification, StatementVerdict, SupportCorrection, VerificationSkip,
    VerificationState,
};
use super::judgment::ReadPassage;
use crate::spaces::db;

/// An assistant message as verification reads it.
pub(super) struct StoredAnswer {
    pub message_id: String,
    pub conversation_id: String,
    pub space_id: String,
    pub run_id: String,
    pub text: String,
}

/// The completed answers of a Run that cite passages. Only those can be verified.
pub(super) async fn cited_answers(
    connection: &mut SqliteConnection,
    run_id: &str,
) -> Result<Vec<StoredAnswer>, String> {
    let rows = sqlx::query(&format!(
        "{ANSWER} WHERE json_extract(m.content,'$.runId')=$1 AND m.role='assistant'
         AND json_array_length(m.content,'$.citations') > 0
         AND NOT EXISTS(SELECT 1 FROM spaces_answer_verifications v WHERE v.message_id=m.id)
         ORDER BY m.created_at, m.rowid"
    ))
    .bind(run_id)
    .fetch_all(connection)
    .await
    .map_err(db::sql_error)?;
    rows.iter().map(answer).collect()
}

pub(super) async fn load_answer(
    connection: &mut SqliteConnection,
    message_id: &str,
) -> Result<StoredAnswer, String> {
    let row = sqlx::query(&format!("{ANSWER} WHERE m.id=$1 AND m.role='assistant'"))
        .bind(message_id)
        .fetch_optional(connection)
        .await
        .map_err(db::sql_error)?
        .ok_or("answer-verification-missing: the answer no longer exists")?;
    answer(&row)
}

/// The answer text is its first part, the text its citation markers were resolved against.
const ANSWER: &str =
    "SELECT m.id,m.conversation_id,c.space_id,json_extract(m.content,'$.runId') AS run_id,
    COALESCE(json_extract(m.content,'$.parts[0].text'),'') AS text
    FROM spaces_conversation_messages m JOIN spaces_conversations c ON c.id=m.conversation_id";

/// The passages the answer's Run read, in citation order, exactly as the Run read them.
pub(super) async fn read_passages(
    connection: &mut SqliteConnection,
    run_id: &str,
) -> Result<Vec<ReadPassage>, String> {
    let rows = sqlx::query(
        "SELECT ordinal,chunk_id,node_id,node_title,source_revision,locator,quote
         FROM spaces_run_citations WHERE run_id=$1 ORDER BY ordinal",
    )
    .bind(run_id)
    .fetch_all(connection)
    .await
    .map_err(db::sql_error)?;
    rows.iter()
        .map(|row| {
            let locator: String = row.try_get("locator").map_err(db::sql_error)?;
            Ok(ReadPassage {
                citation: row.try_get::<i64, _>("ordinal").map_err(db::sql_error)? as u32,
                chunk_id: row.try_get("chunk_id").map_err(db::sql_error)?,
                node_id: row.try_get("node_id").map_err(db::sql_error)?,
                node_title: row.try_get("node_title").map_err(db::sql_error)?,
                source_revision: row.try_get("source_revision").map_err(db::sql_error)?,
                locator: serde_json::from_str(&locator).map_err(db::json_error)?,
                text: row.try_get("quote").map_err(db::sql_error)?,
            })
        })
        .collect()
}

fn answer(row: &SqliteRow) -> Result<StoredAnswer, String> {
    Ok(StoredAnswer {
        message_id: row.try_get("id").map_err(db::sql_error)?,
        conversation_id: row.try_get("conversation_id").map_err(db::sql_error)?,
        space_id: row.try_get("space_id").map_err(db::sql_error)?,
        run_id: row
            .try_get::<Option<String>, _>("run_id")
            .map_err(db::sql_error)?
            .ok_or("answer-verification-invalid: the answer has no Run")?,
        text: row.try_get("text").map_err(db::sql_error)?,
    })
}

/// Records that the answer waits for verification. A message is verified once; asking again
/// keeps the existing verification.
pub(super) async fn insert_pending(
    tx: &mut Transaction<'_, Sqlite>,
    answer: &StoredAnswer,
    now: i64,
) -> Result<bool, String> {
    let executed = sqlx::query(
        "INSERT INTO spaces_answer_verifications(message_id,space_id,answer_run_id,state,created_at)
         VALUES($1,$2,$3,'pending',$4) ON CONFLICT(message_id) DO NOTHING",
    )
    .bind(&answer.message_id)
    .bind(&answer.space_id)
    .bind(&answer.run_id)
    .bind(now)
    .execute(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    Ok(executed.rows_affected() == 1)
}

/// Puts a settled verification back to pending so it runs again, keeping the person's corrections.
pub(super) async fn reset(
    tx: &mut Transaction<'_, Sqlite>,
    message_id: &str,
) -> Result<(), String> {
    let executed = sqlx::query(
        "UPDATE spaces_answer_verifications SET state='pending',skip=NULL,error=NULL,completed_at=NULL,
         verification_run_id=NULL WHERE message_id=$1 AND state IN ('completed','failed','skipped')",
    )
    .bind(message_id)
    .execute(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    db::require_one(
        executed,
        "answer-verification-busy: the answer is being verified",
    )
}

pub(super) async fn assign_run(
    tx: &mut Transaction<'_, Sqlite>,
    message_id: &str,
    run_id: &str,
) -> Result<(), String> {
    let executed = sqlx::query(
        "UPDATE spaces_answer_verifications SET verification_run_id=$1 WHERE message_id=$2 AND state='pending'",
    )
    .bind(run_id)
    .bind(message_id)
    .execute(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    db::require_one(
        executed,
        "answer-verification-busy: the answer is being verified",
    )
}

/// Claims the verification for the Run that executes it. A retried attempt of the same Run claims
/// it again after a failed one; another Run never does.
pub(super) async fn start(
    connection: &mut SqliteConnection,
    message_id: &str,
    run_id: &str,
) -> Result<(), String> {
    let executed = sqlx::query(
        "UPDATE spaces_answer_verifications SET state='running',error=NULL,completed_at=NULL
         WHERE message_id=$1 AND verification_run_id=$2 AND state IN ('pending','running','failed')",
    )
    .bind(message_id)
    .bind(run_id)
    .execute(connection)
    .await
    .map_err(db::sql_error)?;
    db::require_one(
        executed,
        "answer-verification-stale: another Run verifies this answer",
    )
}

pub(super) struct Outcome<'a> {
    pub state: VerificationState,
    pub skip: Option<VerificationSkip>,
    pub statements: &'a [AnswerStatement],
    pub verdicts: &'a [StatementVerdict],
    pub truncated: bool,
    pub error: Option<&'a str>,
}

/// Settles the verification the Run claimed; a Run that lost its claim writes nothing.
pub(super) async fn settle(
    connection: &mut SqliteConnection,
    message_id: &str,
    run_id: Option<&str>,
    outcome: Outcome<'_>,
    now: i64,
) -> Result<(), String> {
    sqlx::query(
        "UPDATE spaces_answer_verifications SET state=$1,skip=$2,statements=$3,verdicts=$4,truncated=$5,
         error=$6,completed_at=$7
         WHERE message_id=$8 AND state IN ('pending','running') AND verification_run_id IS $9",
    )
    .bind(outcome.state.as_str())
    .bind(outcome.skip.map(VerificationSkip::as_str))
    .bind(sqlx::types::Json(outcome.statements))
    .bind(sqlx::types::Json(outcome.verdicts))
    .bind(outcome.truncated)
    .bind(outcome.error)
    .bind(now)
    .bind(message_id)
    .bind(run_id)
    .execute(connection)
    .await
    .map_err(db::sql_error)?;
    Ok(())
}

pub(super) async fn correct(
    tx: &mut Transaction<'_, Sqlite>,
    message_id: &str,
    correction: &SupportCorrection,
) -> Result<(), String> {
    let mut corrections = load(tx, message_id)
        .await?
        .ok_or("answer-verification-missing: the answer is not verified")?
        .corrections;
    corrections.retain(|existing| existing.statement != correction.statement);
    corrections.push(correction.clone());
    sqlx::query("UPDATE spaces_answer_verifications SET corrections=$1 WHERE message_id=$2")
        .bind(sqlx::types::Json(&corrections))
        .bind(message_id)
        .execute(&mut **tx)
        .await
        .map_err(db::sql_error)?;
    Ok(())
}

pub(super) async fn load(
    connection: &mut SqliteConnection,
    message_id: &str,
) -> Result<Option<AnswerVerification>, String> {
    let row = sqlx::query(
        "SELECT message_id,state,skip,statements,verdicts,corrections,truncated,error,completed_at
         FROM spaces_answer_verifications WHERE message_id=$1",
    )
    .bind(message_id)
    .fetch_optional(connection)
    .await
    .map_err(db::sql_error)?;
    row.as_ref().map(decode).transpose()
}

fn decode(row: &SqliteRow) -> Result<AnswerVerification, String> {
    fn json<T: serde::de::DeserializeOwned>(row: &SqliteRow, column: &str) -> Result<T, String> {
        let raw: String = row.try_get(column).map_err(db::sql_error)?;
        serde_json::from_str(&raw).map_err(db::json_error)
    }
    fn label<T: serde::de::DeserializeOwned>(value: Option<String>) -> Result<Option<T>, String> {
        value
            .map(|value| serde_json::from_value(serde_json::Value::String(value)))
            .transpose()
            .map_err(db::json_error)
    }
    Ok(AnswerVerification {
        message_id: row.try_get("message_id").map_err(db::sql_error)?,
        state: label(Some(row.try_get("state").map_err(db::sql_error)?))?
            .ok_or("answer-verification-invalid: state is missing")?,
        skip: label(row.try_get("skip").map_err(db::sql_error)?)?,
        statements: json(row, "statements")?,
        verdicts: json(row, "verdicts")?,
        corrections: json(row, "corrections")?,
        truncated: row.try_get("truncated").map_err(db::sql_error)?,
        error: row.try_get("error").map_err(db::sql_error)?,
        completed_at: row.try_get("completed_at").map_err(db::sql_error)?,
    })
}

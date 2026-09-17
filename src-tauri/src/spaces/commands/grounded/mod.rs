pub(crate) mod answer;
mod answer_context;
mod answer_request;
mod contracts;
pub(crate) mod thread_lifecycle;

use serde_json::Value;
use tauri::{AppHandle, Runtime};
use uuid::Uuid;

use crate::spaces::{db, knowledge::grounding};
use contracts::{GroundedExchangeMessageRequest, GroundedExchangeRequest};

#[cfg(test)]
mod tests;

pub async fn commit_exchange<R: Runtime>(
    app: &AppHandle<R>,
    request: GroundedExchangeRequest,
) -> Result<Value, String> {
    validate_message(&request.user)?;
    validate_message(&request.assistant)?;
    let now = crate::clock::unix_time_millis()?;
    let pool = db::pool(app).await?;
    let mut tx = db::begin_write(pool).await?;

    let (thread_id, contract) = resolve_or_create_thread(
        &mut tx,
        &request.space_id,
        request.thread_id.as_deref(),
        request.new_thread.as_ref(),
        now,
    )
    .await?;

    grounding::validate_current_citations(&mut tx, &contract, &request.user.citations).await?;
    grounding::validate_current_citations(&mut tx, &contract, &request.assistant.citations).await?;
    insert_message(&mut tx, &thread_id, "user", &request.user, now).await?;
    let assistant_created_at = now.saturating_add(1);
    insert_message(
        &mut tx,
        &thread_id,
        "assistant",
        &request.assistant,
        assistant_created_at,
    )
    .await?;
    sqlx::query("UPDATE spaces_ai_threads SET updated_at=$1 WHERE id=$2")
        .bind(assistant_created_at)
        .bind(&thread_id)
        .execute(&mut *tx)
        .await
        .map_err(db::sql_error)?;
    tx.commit().await.map_err(db::sql_error)?;

    Ok(serde_json::json!({
        "threadId": thread_id,
        "userCreatedAt": now,
        "assistantCreatedAt": assistant_created_at,
    }))
}

fn validate_message(message: &GroundedExchangeMessageRequest) -> Result<(), String> {
    if message.id.trim().is_empty() || message.id.encode_utf16().count() > 128 {
        return Err("Grounded AI message ID must be between 1 and 128 UTF-16 code units".into());
    }
    grounding::validate_message_metadata(
        &message.content,
        message.provider.as_deref(),
        message.model.as_deref(),
    )
}

async fn insert_message(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    thread_id: &str,
    role: &str,
    message: &GroundedExchangeMessageRequest,
    created_at: i64,
) -> Result<(), String> {
    let citations = serde_json::to_string(&message.citations).map_err(db::json_error)?;
    sqlx::query(
        "INSERT INTO spaces_ai_messages(id,thread_id,role,content,citations,provider,model,created_at) \
         VALUES($1,$2,$3,$4,$5,$6,$7,$8)",
    )
    .bind(&message.id)
    .bind(thread_id)
    .bind(role)
    .bind(&message.content)
    .bind(citations)
    .bind(message.provider.as_deref())
    .bind(message.model.as_deref())
    .bind(created_at)
    .execute(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    Ok(())
}

async fn resolve_or_create_thread(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    space_id: &str,
    thread_id: Option<&str>,
    new_thread: Option<&contracts::GroundedThreadCreationRequest>,
    now: i64,
) -> Result<(String, grounding::GroundedThreadContract), String> {
    match (thread_id, new_thread) {
        (Some(thread_id), None) => {
            let contract = grounding::thread_contract(tx, thread_id).await?;
            if contract.space_id != space_id {
                return Err("Knowledge thread belongs to another Space".into());
            }
            Ok((thread_id.to_owned(), contract))
        }
        (None, Some(new_thread)) => {
            grounding::validate_title(&new_thread.title)?;
            grounding::validate_new_scope_nodes(
                tx,
                space_id,
                &new_thread.scope,
                &new_thread.node_ids,
            )
            .await?;
            let id = Uuid::new_v4().to_string();
            let scope_node_ids =
                serde_json::to_string(&new_thread.node_ids).map_err(db::json_error)?;
            sqlx::query(
                "INSERT INTO spaces_ai_threads(id,space_id,scope,scope_node_ids,title,created_at,updated_at) \
                 VALUES($1,$2,$3,$4,$5,$6,$6)",
            )
            .bind(&id)
            .bind(space_id)
            .bind(&new_thread.scope)
            .bind(scope_node_ids)
            .bind(&new_thread.title)
            .bind(now)
            .execute(&mut **tx)
            .await
            .map_err(db::sql_error)?;
            let contract = grounding::GroundedThreadContract {
                space_id: space_id.to_owned(),
                scope: new_thread.scope.clone(),
                node_ids: new_thread.node_ids.clone(),
            };
            Ok((id, contract))
        }
        _ => Err(
            "Grounded exchange must target either one existing thread or one new thread contract"
                .into(),
        ),
    }
}

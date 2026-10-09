use std::path::Path;

use sqlx::{Row, SqliteConnection, SqlitePool};

use crate::spaces::{conversations::contracts::ConversationAttachment, db};

use super::super::contracts::{ConversationDraft, SaveConversationDraftRequest};

const MAX_DRAFT_JSON_BYTES: usize = 1_000_000;
const MAX_NODE_ID_CHARS: usize = 128;
const MAX_SOURCE_REVISION_CHARS: usize = 2_048;
const MAX_ATTACHMENT_ROLE_CHARS: usize = 128;
const MAX_ATTACHMENT_PATH_CHARS: usize = 32_768;

pub async fn load_draft(
    app: &tauri::AppHandle,
    space_id: &str,
    conversation_id: &str,
) -> Result<Option<ConversationDraft>, String> {
    let mut connection = crate::spaces::db::pool(app)
        .await?
        .acquire()
        .await
        .map_err(db::sql_error)?;
    let row = sqlx::query(
        "SELECT prompt,attachments,extensions,reacquire_file_names FROM spaces_conversation_drafts
         WHERE space_id=$1 AND conversation_id=$2",
    )
    .bind(space_id)
    .bind(conversation_id)
    .fetch_optional(&mut *connection)
    .await
    .map_err(db::sql_error)?;
    let Some(row) = row else {
        return Ok(None);
    };
    let draft = parse_draft(space_id, conversation_id, row)?;
    let (attachments, _) =
        prepare_attachments(app, &draft.attachments, &draft.reacquire_file_names, false)?;
    validate_identity(&mut connection, space_id, conversation_id).await?;
    validate_node_attachments(&mut connection, space_id, &attachments).await?;
    Ok(Some(draft))
}

pub async fn save_draft(
    app: &tauri::AppHandle,
    request: SaveConversationDraftRequest,
) -> Result<ConversationDraft, String> {
    let pool = crate::spaces::db::pool(app).await?;
    if request.prompt.chars().count() > 1_000_000
        || request.attachments.len() > 32
        || request.reacquire_file_names.len() > 32
    {
        return Err("conversation-draft-invalid".into());
    }
    let (attachments, reacquire_file_names) = prepare_attachments(
        app,
        &request.attachments,
        &request.reacquire_file_names,
        true,
    )?;
    let attachments_json = serialize_attachments(&attachments)?;
    let extensions_json = serde_json::to_string(&request.extensions).map_err(db::json_error)?;
    let reacquire_json = serde_json::to_string(&reacquire_file_names).map_err(db::json_error)?;
    if extensions_json.len() > MAX_DRAFT_JSON_BYTES || reacquire_json.len() > MAX_DRAFT_JSON_BYTES {
        return Err("conversation-draft-invalid".into());
    }
    let mut tx = db::begin_write(pool).await?;
    validate_identity(&mut tx, &request.space_id, &request.conversation_id).await?;
    validate_node_attachments(&mut tx, &request.space_id, &attachments).await?;
    sqlx::query(
        "INSERT INTO spaces_conversation_drafts
         (space_id,conversation_id,prompt,attachments,extensions,reacquire_file_names,updated_at)
         VALUES($1,$2,$3,$4,$5,$6,$7)
         ON CONFLICT(space_id,conversation_id) DO UPDATE SET prompt=excluded.prompt,
         attachments=excluded.attachments,extensions=excluded.extensions,
         reacquire_file_names=excluded.reacquire_file_names,updated_at=excluded.updated_at",
    )
    .bind(&request.space_id)
    .bind(&request.conversation_id)
    .bind(&request.prompt)
    .bind(&attachments_json)
    .bind(&extensions_json)
    .bind(&reacquire_json)
    .bind(crate::clock::unix_time_millis()?)
    .execute(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    tx.commit().await.map_err(db::sql_error)?;
    Ok(ConversationDraft {
        space_id: request.space_id,
        conversation_id: request.conversation_id,
        prompt: request.prompt,
        attachments,
        extensions: request.extensions,
        reacquire_file_names,
    })
}

pub async fn remove_draft(
    pool: &SqlitePool,
    space_id: &str,
    conversation_id: &str,
) -> Result<(), String> {
    sqlx::query("DELETE FROM spaces_conversation_drafts WHERE space_id=$1 AND conversation_id=$2")
        .bind(space_id)
        .bind(conversation_id)
        .execute(pool)
        .await
        .map_err(db::sql_error)?;
    Ok(())
}

async fn validate_identity(
    connection: &mut SqliteConnection,
    space_id: &str,
    conversation_id: &str,
) -> Result<(), String> {
    let valid: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM spaces WHERE id=$1 AND archived_at IS NULL)
         AND ($2='' OR EXISTS(SELECT 1 FROM spaces_conversations WHERE id=$2 AND space_id=$1))",
    )
    .bind(space_id)
    .bind(conversation_id)
    .fetch_one(&mut *connection)
    .await
    .map_err(db::sql_error)?;
    if valid {
        Ok(())
    } else {
        Err("conversation-draft-unavailable".into())
    }
}

fn prepare_attachments(
    app: &tauri::AppHandle,
    attachments: &[ConversationAttachment],
    reacquire_file_names: &[String],
    allow_external_files: bool,
) -> Result<(Vec<ConversationAttachment>, Vec<String>), String> {
    if attachments.len() > 32 || reacquire_file_names.len() > 32 {
        return Err("conversation-draft-invalid".into());
    }
    let mut durable = Vec::new();
    if reacquire_file_names.iter().any(|name| {
        name.trim().is_empty()
            || name.chars().count() > 255
            || Path::new(name).file_name().and_then(|value| value.to_str()) != Some(name.as_str())
    }) {
        return Err("conversation-draft-attachment-invalid".into());
    }
    let mut reacquire = Vec::new();
    for name in reacquire_file_names {
        if !reacquire.contains(name) {
            reacquire.push(name.clone());
        }
    }
    for attachment in attachments {
        validate_attachment(attachment)?;
        match attachment {
            ConversationAttachment::Node { .. } => durable.push(attachment.clone()),
            ConversationAttachment::File {
                path,
                source_revision,
            } => {
                let source = Path::new(path);
                if let Ok(managed) = crate::spaces::assets::authorize_spaces_asset(app, source) {
                    crate::file_inspection::verify_content_revision(&managed, source_revision)?;
                    durable.push(attachment.clone());
                } else {
                    if !allow_external_files {
                        return Err("conversation-draft-attachment-invalid".into());
                    }
                    let authorized = super::super::upload::authorize(app, path)
                        .map_err(|error| error.to_string())?;
                    super::super::upload::classify(&authorized)
                        .map_err(|error| error.to_string())?;
                    crate::file_inspection::verify_content_revision(&authorized, source_revision)?;
                    let name = authorized
                        .file_name()
                        .and_then(|value| value.to_str())
                        .ok_or("conversation-draft-attachment-invalid")?;
                    if !reacquire.iter().any(|existing| existing == name) {
                        reacquire.push(name.to_owned());
                    }
                }
            }
        }
    }
    if reacquire.len() > 32 {
        return Err("conversation-draft-invalid".into());
    }
    Ok((durable, reacquire))
}

pub(super) fn serialize_attachments(
    attachments: &[ConversationAttachment],
) -> Result<String, String> {
    let serialized = serde_json::to_string(attachments).map_err(db::json_error)?;
    if serialized.len() > MAX_DRAFT_JSON_BYTES {
        Err("conversation-draft-invalid".into())
    } else {
        Ok(serialized)
    }
}

pub(super) fn validate_attachment(attachment: &ConversationAttachment) -> Result<(), String> {
    let valid = match attachment {
        ConversationAttachment::Node {
            node_id,
            source_revision,
            role,
            locator,
        } => {
            let fields_valid = valid_text(node_id, MAX_NODE_ID_CHARS)
                && valid_text(source_revision, MAX_SOURCE_REVISION_CHARS)
                && valid_text(role, MAX_ATTACHMENT_ROLE_CHARS);
            fields_valid
                && locator.as_ref().is_none_or(|value| {
                    crate::spaces::knowledge::locator::validate(value, Some(source_revision))
                        .is_ok()
                })
        }
        ConversationAttachment::File {
            path,
            source_revision,
        } => {
            valid_text(path, MAX_ATTACHMENT_PATH_CHARS)
                && valid_text(source_revision, MAX_SOURCE_REVISION_CHARS)
        }
    };
    if valid {
        Ok(())
    } else {
        Err("conversation-draft-attachment-invalid".into())
    }
}

fn valid_text(value: &str, max_chars: usize) -> bool {
    !value.trim().is_empty() && value.chars().count() <= max_chars
}

async fn validate_node_attachments(
    connection: &mut SqliteConnection,
    space_id: &str,
    attachments: &[ConversationAttachment],
) -> Result<(), String> {
    for attachment in attachments {
        let ConversationAttachment::Node {
            node_id,
            source_revision,
            ..
        } = attachment
        else {
            continue;
        };
        let current: Option<String> = sqlx::query_scalar(
            "SELECT COALESCE(json_extract(data,'$.knowledgeRevision'),json_extract(data,'$.sourceRevision'))
             FROM spaces_nodes WHERE id=$1 AND space_id=$2 AND deleted_at IS NULL",
        )
        .bind(node_id)
        .bind(space_id)
        .fetch_optional(&mut *connection)
        .await
        .map_err(db::sql_error)?;
        if current.as_deref() != Some(source_revision) {
            return Err("conversation-draft-attachment-stale".into());
        }
    }
    Ok(())
}

pub(super) fn parse_draft(
    space_id: &str,
    conversation_id: &str,
    row: sqlx::sqlite::SqliteRow,
) -> Result<ConversationDraft, String> {
    let prompt: String = row.try_get("prompt").map_err(db::sql_error)?;
    let extensions: String = row.try_get("extensions").map_err(db::sql_error)?;
    let attachments_json: String = row.try_get("attachments").map_err(db::sql_error)?;
    let reacquire_json: String = row.try_get("reacquire_file_names").map_err(db::sql_error)?;
    if prompt.chars().count() > 1_000_000
        || extensions.len() > MAX_DRAFT_JSON_BYTES
        || attachments_json.len() > MAX_DRAFT_JSON_BYTES
        || reacquire_json.len() > MAX_DRAFT_JSON_BYTES
    {
        return Err("conversation-draft-invalid".into());
    }
    let attachments: Vec<ConversationAttachment> =
        serde_json::from_str(&attachments_json).map_err(db::json_error)?;
    Ok(ConversationDraft {
        space_id: space_id.to_owned(),
        conversation_id: conversation_id.to_owned(),
        prompt,
        attachments,
        extensions: serde_json::from_str(&extensions).map_err(db::json_error)?,
        reacquire_file_names: serde_json::from_str(&reacquire_json).map_err(db::json_error)?,
    })
}

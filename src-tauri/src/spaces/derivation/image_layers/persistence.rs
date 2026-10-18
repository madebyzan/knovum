use serde_json::{json, Value};
use sqlx::{Sqlite, Transaction};
use tauri::AppHandle;

use super::{assets::PreparedLayer, contracts, source};
use crate::foundation::app_error::AppError;
use crate::spaces::{
    db,
    derivation::{
        action_execution::ExecutionAuthority, failure, inputs, pending_output,
        CompletedDerivationOutput,
    },
    knowledge::model::revision,
};

/// Node titles are validated in UTF-16 code units, which is the budget a layer title shares with
/// its trimmed source title.
const MAX_TITLE: usize = 512;

/// Reserves one pending node per discovered layer. The layers are already on disk, so each node is
/// reserved with the asset it will publish and the pixel size it will report, which lets the set be
/// laid out at the size it keeps. The node the Run reserved up front becomes the first layer.
pub(super) async fn reserve(
    app: &AppHandle,
    request: &contracts::SplitImageLayersRequest,
    source: &source::SourceImage,
    layers: &[PreparedLayer],
    authority: &ExecutionAuthority,
) -> Result<(), AppError> {
    let outputs = layers
        .iter()
        .enumerate()
        .map(|(index, layer)| {
            let title = layer_title(&source.title, index);
            pending_output::PendingOutput {
                node_id: contracts::layer_node_id(request, index),
                kind: "image",
                data: json!({
                    "kind": "image", "title": title, "fileName": format!("{title}.png"),
                    "assetPath": layer.path.to_string_lossy(), "processingState": "processing",
                    "intrinsicWidth": layer.width, "intrinsicHeight": layer.height
                }),
            }
        })
        .collect::<Vec<_>>();
    for output in &outputs {
        pending_output::validate_separate_asset(&output.data, &source.asset_path)?;
    }
    pending_output::reserve(
        app,
        &request.space_id,
        outputs,
        std::slice::from_ref(&request.source_node_id),
        super::LINEAGE,
        authority,
    )
    .await
}

/// Opens one derivation per reserved output so each layer carries its own lineage.
pub(super) async fn start(
    app: &AppHandle,
    request: &contracts::SplitImageLayersRequest,
    count: usize,
    authority: &ExecutionAuthority,
) -> Result<(), AppError> {
    let pool = db::pool(app).await?;
    let mut tx = db::begin_write(pool).await?;
    authority.require_active(&mut *tx).await?;
    let now = crate::foundation::clock::unix_time_millis()?;
    for index in 0..count {
        let derivation_id = contracts::derivation_id(&request.id, index);
        sqlx::query("INSERT INTO spaces_knowledge_derivations(id,space_id,output_node_id,action_kind,action_version,engine,status,created_at,updated_at) VALUES($1,$2,$3,'split-image-layers',1,'local:image-layers','running',$4,$4)")
            .bind(&derivation_id).bind(&request.space_id)
            .bind(contracts::layer_node_id(request, index))
            .bind(now).execute(&mut *tx).await.map_err(db::sql_error)?;
        inputs::insert(
            &mut tx,
            &derivation_id,
            &[inputs::DerivationInput {
                node_id: &request.source_node_id,
                role: "primary",
                source_revision: &request.source_revision,
                revision_basis: inputs::RevisionBasis::Rendered,
            }],
        )
        .await?;
    }
    tx.commit()
        .await
        .map_err(db::sql_error)
        .map_err(AppError::from)
}

pub(super) async fn complete(
    app: &AppHandle,
    request: &contracts::SplitImageLayersRequest,
    layers: &[PreparedLayer],
    authority: &ExecutionAuthority,
) -> Result<Vec<CompletedDerivationOutput>, AppError> {
    let pool = db::pool(app).await?;
    let mut tx = db::begin_write(pool).await?;
    authority.require_active(&mut *tx).await?;
    source::load(&mut *tx, request).await?;
    let now = crate::foundation::clock::unix_time_millis()?;
    let enrichment_target = crate::spaces::assets::enrichment::jobs::intent::selection(app)?;
    let mut completed = Vec::with_capacity(layers.len());
    for (index, layer) in layers.iter().enumerate() {
        let output_id = contracts::layer_node_id(request, index);
        let mut data =
            load_pending_output(&mut tx, &contracts::derivation_id(&request.id, index)).await?;
        pending_output::validate_unpublished_asset(&data)?;
        apply_layer_content(&mut data, layer, now)?;
        crate::spaces::knowledge::validation::validate_node(
            &mut tx,
            &request.space_id,
            "image",
            &data,
        )
        .await?;
        crate::spaces::derivation::persist_output_node(
            &mut tx,
            &output_id,
            &data,
            now,
            enrichment_target.as_ref(),
        )
        .await?;
        sqlx::query("UPDATE spaces_knowledge_derivations SET status='completed',completed_at=$1,updated_at=$1 WHERE id=$2 AND status='running'")
            .bind(now).bind(contracts::derivation_id(&request.id, index)).execute(&mut *tx).await.map_err(db::sql_error)?;
        completed.push(CompletedDerivationOutput {
            node_id: output_id,
            data,
        });
    }
    sqlx::query("UPDATE spaces SET updated_at=$1 WHERE id=$2")
        .bind(now)
        .bind(&request.space_id)
        .execute(&mut *tx)
        .await
        .map_err(db::sql_error)?;
    tx.commit()
        .await
        .map_err(db::sql_error)
        .map_err(AppError::from)?;
    Ok(completed)
}

/// Settles reserved outputs whose derivations never opened.
pub(super) async fn fail_reserved(
    app: &AppHandle,
    request: &contracts::SplitImageLayersRequest,
    count: usize,
) -> Result<(), String> {
    for index in 0..count {
        failure::fail_pending(
            app,
            &request.space_id,
            &contracts::layer_node_id(request, index),
            failure::PendingNodeState::Processing,
        )
        .await?;
    }
    Ok(())
}

pub(super) async fn fail(
    app: &AppHandle,
    request: &contracts::SplitImageLayersRequest,
    count: usize,
    error: &str,
) -> Result<(), String> {
    let pool = db::pool(app).await?;
    let ids = (0..count)
        .map(|index| contracts::derivation_id(&request.id, index))
        .collect::<Vec<_>>();
    failure::fail_messages(pool, &ids, error).await
}

async fn load_pending_output(
    tx: &mut Transaction<'_, Sqlite>,
    derivation_id: &str,
) -> Result<Value, AppError> {
    let encoded = sqlx::query_scalar::<_, String>(
        "SELECT n.data FROM spaces_knowledge_derivations d JOIN spaces_nodes n ON n.id=d.output_node_id WHERE d.id=$1 AND d.status='running' AND n.kind='image' AND n.deleted_at IS NULL",
    )
    .bind(derivation_id).fetch_optional(&mut **tx).await.map_err(db::sql_error)?
    .ok_or("Image layer output is no longer pending")?;
    serde_json::from_str(&encoded)
        .map_err(db::json_error)
        .map_err(AppError::from)
}

fn apply_layer_content(
    payload: &mut Value,
    layer: &PreparedLayer,
    now: i64,
) -> Result<(), AppError> {
    let object = payload
        .as_object_mut()
        .ok_or("Image layer output data is invalid")?;
    object.insert(
        "assetPath".into(),
        layer.path.to_string_lossy().into_owned().into(),
    );
    object.insert("sourceRevision".into(), layer.digest.clone().into());
    object.insert("size".into(), (layer.size as u64).into());
    object.insert("modifiedAt".into(), now.into());
    object.insert("intrinsicWidth".into(), layer.width.into());
    object.insert("intrinsicHeight".into(), layer.height.into());
    object.remove("processingState");
    revision::apply(payload)?;
    Ok(())
}

/// Backdrop first, then each subject in the order the stack composites. Node titles are bounded,
/// so a long source title is trimmed instead of rejected on publication.
fn layer_title(source: &str, index: usize) -> String {
    let suffix = format!("-layer-{}", index + 1);
    let budget = MAX_TITLE.saturating_sub(suffix.encode_utf16().count());
    let mut title = String::new();
    let mut used = 0;
    for character in source.chars() {
        used += character.len_utf16();
        if used > budget {
            break;
        }
        title.push(character);
    }
    title.push_str(&suffix);
    title
}

#[cfg(test)]
#[path = "tests/persistence.rs"]
mod tests;

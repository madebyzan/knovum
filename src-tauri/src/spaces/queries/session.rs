use crate::command_error::CommandError;
use serde::Serialize;
use tauri::{AppHandle, Runtime};
use ts_rs::TS;

use super::{nodes, relations, scenes};
use crate::spaces::{db, structure};

#[derive(Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub(crate) struct SpaceSessionProjection {
    nodes: Vec<nodes::SpaceNodeRecord>,
    scenes: Vec<scenes::SpaceSceneRecord>,
    relations: Vec<relations::SpaceRelationRecord>,
    #[ts(type = "unknown")]
    edit_snapshot: structure::SpaceEditSnapshotResponse,
}

#[tauri::command]
pub(crate) async fn load_space_session_projection<R: Runtime>(
    app: AppHandle<R>,
    space_id: String,
) -> Result<SpaceSessionProjection, CommandError> {
    let pool = db::pool(&app).await?;
    let mut tx = pool.begin().await.map_err(db::sql_error)?;
    let (snapshot, edit_revision) =
        structure::repository::load_in_transaction(&mut tx, &space_id).await?;
    let nodes = nodes::list_in_transaction(&mut tx, &space_id).await?;
    let scenes = scenes::list_in_transaction(&mut tx, &space_id).await?;
    let relations = relations::list_in_transaction(&mut tx, &space_id).await?;
    tx.commit().await.map_err(db::sql_error)?;
    Ok(SpaceSessionProjection {
        nodes,
        scenes,
        relations,
        edit_snapshot: structure::SpaceEditSnapshotResponse {
            edit_revision,
            structure: snapshot.structure,
            layouts: snapshot.layouts,
            relation_styles: snapshot.relation_styles,
        },
    })
}

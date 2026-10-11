use serde::{Deserialize, Serialize};
use sqlx::{Row, Sqlite, Transaction};

use crate::foundation::app_error::AppError;
use crate::spaces::db;

#[derive(Clone, Deserialize, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct SpaceEditorResource {
    pub space_id: String,
    pub node_id: String,
}

/// A transaction may change layout, but must preserve every live editor resource's owner.
pub(crate) struct EditorResourceProtection(Vec<SpaceEditorResource>);

impl EditorResourceProtection {
    pub(crate) async fn capture(
        tx: &mut Transaction<'_, Sqlite>,
        resources: &[SpaceEditorResource],
    ) -> Result<Self, AppError> {
        for resource in resources {
            super::validation::validate_id("Editor Space ID", &resource.space_id)?;
            super::validation::validate_id("Editor node ID", &resource.node_id)?;
        }
        Ok(Self(live_resources(tx, resources).await?))
    }

    pub(crate) async fn ensure_preserved(
        self,
        tx: &mut Transaction<'_, Sqlite>,
    ) -> Result<(), AppError> {
        if live_resources(tx, &self.0).await?.len() != self.0.len() {
            return Err(AppError::new(
                "space_editor_resource_protected",
                "Close the editor tab before removing or transferring its resource",
            ));
        }
        Ok(())
    }
}

async fn live_resources(
    tx: &mut Transaction<'_, Sqlite>,
    resources: &[SpaceEditorResource],
) -> Result<Vec<SpaceEditorResource>, AppError> {
    let rows = sqlx::query(
        "SELECT DISTINCT n.space_id,n.id FROM json_each($1) r \
         JOIN spaces_nodes n ON n.id=json_extract(r.value,'$.nodeId') \
           AND n.space_id=json_extract(r.value,'$.spaceId') \
         WHERE n.deleted_at IS NULL \
         AND EXISTS (SELECT 1 FROM spaces_structure_nodes s \
           WHERE s.space_id=n.space_id AND s.node_id=n.id AND s.deleted_at IS NULL)",
    )
    .bind(sqlx::types::Json(resources))
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    rows.into_iter()
        .map(|row| {
            Ok(SpaceEditorResource {
                space_id: row.try_get("space_id").map_err(db::sql_error)?,
                node_id: row.try_get("id").map_err(db::sql_error)?,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests;

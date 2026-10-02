use crate::spaces::db;
use crate::spaces::structure::{repository::SpaceStructureSnapshot, RelationStyle};
use sqlx::{Row, Sqlite, Transaction};
use std::collections::HashMap;
use uuid::Uuid;

pub(super) async fn clone_nodes(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    ids: &HashMap<String, String>,
    structure_ids: &HashMap<String, String>,
) -> Result<(), String> {
    let now = crate::clock::unix_time_millis()?;
    for (old_id, new_id) in ids {
        let row = sqlx::query("SELECT n.kind,n.object_type_id,n.initial_x,n.initial_y,n.data,t.fields AS object_fields FROM spaces_nodes n LEFT JOIN spaces_object_types t ON t.id=n.object_type_id WHERE n.id=$1 AND n.space_id=$2 AND n.deleted_at IS NULL")
            .bind(old_id).bind(space_id).fetch_optional(&mut **tx).await.map_err(db::sql_error)?
            .ok_or_else(|| format!("Clone content node not found: {old_id}"))?;
        let mut data: serde_json::Value = serde_json::from_str(
            row.try_get::<String, _>("data")
                .map_err(db::sql_error)?
                .as_str(),
        )
        .map_err(db::json_error)?;
        let fields: Option<String> = row.try_get("object_fields").map_err(db::sql_error)?;
        let fields = fields
            .as_deref()
            .map(|raw| serde_json::from_str(raw).map_err(db::json_error))
            .transpose()?;
        super::super::super::node_reference_remap::remap(
            &mut data,
            ids,
            structure_ids,
            fields.as_ref(),
        )?;
        sqlx::query("INSERT INTO spaces_nodes(id,space_id,kind,object_type_id,initial_x,initial_y,data,created_at,updated_at,deleted_at) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$8,NULL)")
            .bind(new_id).bind(space_id)
            .bind(row.try_get::<String,_>("kind").map_err(db::sql_error)?)
            .bind(row.try_get::<Option<String>,_>("object_type_id").map_err(db::sql_error)?)
            .bind(row.try_get::<f64,_>("initial_x").map_err(db::sql_error)?)
            .bind(row.try_get::<f64,_>("initial_y").map_err(db::sql_error)?)
            .bind(serde_json::to_string(&data).map_err(db::json_error)?).bind(now)
            .execute(&mut **tx).await.map_err(db::sql_error)?;
    }
    for new_id in ids.values() {
        let row = sqlx::query("SELECT kind,data FROM spaces_nodes WHERE id=$1 AND space_id=$2")
            .bind(new_id)
            .bind(space_id)
            .fetch_one(&mut **tx)
            .await
            .map_err(db::sql_error)?;
        let kind: String = row.try_get("kind").map_err(db::sql_error)?;
        let raw: String = row.try_get("data").map_err(db::sql_error)?;
        let data = serde_json::from_str(&raw).map_err(db::json_error)?;
        crate::spaces::knowledge::contracts::validate_node(tx, space_id, &kind, &data).await?;
    }
    Ok(())
}

pub(super) async fn clone_relations(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    node_ids: &HashMap<String, String>,
    snapshot: &mut SpaceStructureSnapshot,
) -> Result<(), String> {
    let rows = sqlx::query("SELECT id,source_id,target_id,kind,rationale,status,status_before_invalidation,confidence,actor_kind,actor_id,created_at,updated_at FROM spaces_relations WHERE space_id=$1")
        .bind(space_id).fetch_all(&mut **tx).await.map_err(db::sql_error)?;
    let styles: HashMap<String, _> = snapshot
        .relation_styles
        .iter()
        .map(|style| (style.relation_id.clone(), style.color.clone()))
        .collect();
    for row in rows {
        let source: String = row.try_get("source_id").map_err(db::sql_error)?;
        let target: String = row.try_get("target_id").map_err(db::sql_error)?;
        let (Some(new_source), Some(new_target)) = (node_ids.get(&source), node_ids.get(&target))
        else {
            continue;
        };
        let old_relation_id: String = row.try_get("id").map_err(db::sql_error)?;
        let new_relation_id = Uuid::new_v4().to_string();
        sqlx::query("INSERT INTO spaces_relations(id,space_id,source_id,target_id,kind,rationale,status,status_before_invalidation,confidence,actor_kind,actor_id,created_at,updated_at) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13)")
            .bind(&new_relation_id).bind(space_id).bind(new_source).bind(new_target)
            .bind(row.try_get::<String,_>("kind").map_err(db::sql_error)?)
            .bind(row.try_get::<String,_>("rationale").map_err(db::sql_error)?)
            .bind(row.try_get::<String,_>("status").map_err(db::sql_error)?)
            .bind(row.try_get::<Option<String>,_>("status_before_invalidation").map_err(db::sql_error)?)
            .bind(row.try_get::<Option<f64>,_>("confidence").map_err(db::sql_error)?)
            .bind(row.try_get::<String,_>("actor_kind").map_err(db::sql_error)?)
            .bind(row.try_get::<Option<String>,_>("actor_id").map_err(db::sql_error)?)
            .bind(row.try_get::<i64,_>("created_at").map_err(db::sql_error)?)
            .bind(row.try_get::<i64,_>("updated_at").map_err(db::sql_error)?)
            .execute(&mut **tx).await.map_err(db::sql_error)?;
        if let Some(color) = styles.get(&old_relation_id) {
            snapshot.relation_styles.push(RelationStyle {
                relation_id: new_relation_id,
                color: color.clone(),
            });
        }
    }
    Ok(())
}

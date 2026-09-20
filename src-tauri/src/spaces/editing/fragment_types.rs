use sqlx::Row;
use crate::spaces::{db, knowledge::{events, types}, organization::FragmentObjectType};

pub(super) async fn insert(tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>, space_id: &str, definitions: &[FragmentObjectType], now: i64) -> Result<(), String> {
    for definition in definitions {
        types::validate_custom_type_definition(&definition.id, &definition.name, &definition.icon, &definition.description, &definition.fields)?;
        let existing = sqlx::query("SELECT space_id,name,fields,system FROM spaces_object_types WHERE id=$1").bind(&definition.id).fetch_optional(&mut **tx).await.map_err(db::sql_error)?;
        if let Some(row) = existing {
            let fields: serde_json::Value = serde_json::from_str(&row.try_get::<String,_>("fields").map_err(db::sql_error)?).map_err(db::json_error)?;
            if row.try_get::<Option<String>,_>("space_id").map_err(db::sql_error)?.as_deref() != Some(space_id) || row.try_get::<i64,_>("system").map_err(db::sql_error)? != 0 || row.try_get::<String,_>("name").map_err(db::sql_error)? != definition.name || fields != definition.fields {
                return Err("The destination custom type changed while the fragment was prepared".into());
            }
            continue;
        }
        sqlx::query("INSERT INTO spaces_object_types(id,space_id,name,icon,description,fields,system,created_at,updated_at) VALUES($1,$2,$3,$4,$5,$6,0,$7,$7)")
            .bind(&definition.id).bind(space_id).bind(&definition.name).bind(&definition.icon).bind(&definition.description)
            .bind(serde_json::to_string(&definition.fields).map_err(db::json_error)?).bind(now).execute(&mut **tx).await.map_err(db::sql_error)?;
        events::append(tx, events::EventRecord::new(space_id, events::EventEntity::new("type", &definition.id), "created", events::EventActor::new("human", None), &serde_json::json!({"name": definition.name}), now)).await?;
    }
    Ok(())
}

//! The relations and provenance a cross-Space move carries.
//!
//! A dependency whose ends all move goes with them. One that would join a moving node to a live
//! node left behind would span two Spaces, so the move is refused. One whose other ends are in the
//! source Space's trash — deleted content, kept only so undo there can bring it back — cannot
//! follow a node into another Space: it is severed, as reclamation erases a dependency whose end
//! can no longer be restored, and undoing that deletion restores the node without it.

use std::collections::HashSet;

use sqlx::{Row, Sqlite, Transaction};

use crate::spaces::{db, structure};

const RELATION_SPLIT: &str =
    "Relations cannot be split across Spaces; move both endpoints together";
const PROVENANCE_SPLIT: &str = "Provenance source and target must move together between Spaces";

#[derive(Default)]
pub(super) struct Carried {
    pub moved: Vec<String>,
    pub severed: Vec<String>,
}

pub(super) struct Dependencies {
    pub relations: Carried,
    pub provenance: Carried,
}

/// Reads every dependency of the source Space and how the move carries it, refusing a move that
/// would split one between live nodes.
pub(super) async fn carried(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    moving: &HashSet<&str>,
) -> Result<Dependencies, String> {
    let boundary = Boundary {
        moving,
        trashed: trashed(tx, space_id).await?,
    };
    let mut relations = Carried::default();
    let rows = sqlx::query("SELECT id,source_id,target_id FROM spaces_relations WHERE space_id=$1")
        .bind(space_id)
        .fetch_all(&mut **tx)
        .await
        .map_err(db::sql_error)?;
    for row in rows {
        let ends = [text(&row, "source_id")?, text(&row, "target_id")?];
        relations.include(text(&row, "id")?, boundary.crossing(&ends, RELATION_SPLIT)?);
    }
    let mut provenance = Carried::default();
    let rows = sqlx::query(
        "SELECT p.id,p.source_node_id,p.target_kind,p.target_id, \
         r.source_id AS relation_source,r.target_id AS relation_target \
         FROM spaces_provenance p LEFT JOIN spaces_relations r \
         ON p.target_kind='relation' AND r.id=p.target_id WHERE p.space_id=$1",
    )
    .bind(space_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    for row in rows {
        let ends = provenance_ends(&row)?;
        provenance.include(
            text(&row, "id")?,
            boundary.crossing(&ends, PROVENANCE_SPLIT)?,
        );
    }
    Ok(Dependencies {
        relations,
        provenance,
    })
}

/// Erases the severed dependencies, and the severed relations' styles from the source snapshot
/// that would otherwise write them back.
pub(super) async fn sever(
    tx: &mut Transaction<'_, Sqlite>,
    dependencies: &Dependencies,
    source: &mut structure::repository::SpaceStructureSnapshot,
) -> Result<(), String> {
    let severed = &dependencies.relations.severed;
    source
        .relation_styles
        .retain(|style| !severed.contains(&style.relation_id));
    structure::reclamation::erase_dependencies(tx, severed, &dependencies.provenance.severed).await
}

impl Carried {
    fn include(&mut self, id: String, crossing: Crossing) {
        match crossing {
            Crossing::Stays => {}
            Crossing::Moves => self.moved.push(id),
            Crossing::Severs => self.severed.push(id),
        }
    }
}

#[derive(Debug, PartialEq)]
enum Crossing {
    Stays,
    Moves,
    Severs,
}

struct Boundary<'a> {
    moving: &'a HashSet<&'a str>,
    trashed: HashSet<String>,
}

impl Boundary<'_> {
    fn crossing(&self, ends: &[String], split: &str) -> Result<Crossing, String> {
        let (moving, left): (Vec<&String>, Vec<&String>) = ends
            .iter()
            .partition(|end| self.moving.contains(end.as_str()));
        if moving.is_empty() {
            Ok(Crossing::Stays)
        } else if left.is_empty() {
            Ok(Crossing::Moves)
        } else if left.iter().all(|end| self.trashed.contains(end.as_str())) {
            Ok(Crossing::Severs)
        } else {
            Err(split.into())
        }
    }
}

async fn trashed(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
) -> Result<HashSet<String>, String> {
    let ids: Vec<String> = sqlx::query_scalar(
        "SELECT id FROM spaces_nodes WHERE space_id=$1 AND deleted_at IS NOT NULL",
    )
    .bind(space_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    Ok(ids.into_iter().collect())
}

/// The nodes a provenance row ties together: its source, and its target node or both ends of its
/// target relation.
fn provenance_ends(row: &sqlx::sqlite::SqliteRow) -> Result<Vec<String>, String> {
    let mut ends = vec![text(row, "source_node_id")?];
    if text(row, "target_kind")? == "node" {
        ends.push(text(row, "target_id")?);
    } else {
        for column in ["relation_source", "relation_target"] {
            let end: Option<String> = row.try_get(column).map_err(db::sql_error)?;
            ends.extend(end);
        }
    }
    Ok(ends)
}

fn text(row: &sqlx::sqlite::SqliteRow, column: &str) -> Result<String, String> {
    row.try_get(column).map_err(db::sql_error)
}

#[cfg(test)]
#[path = "tests/dependencies.rs"]
mod tests;

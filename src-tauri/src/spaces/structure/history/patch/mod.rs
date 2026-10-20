//! What one edit changed, expressed as the rows it touched.
//!
//! A record holds the value a row had before the edit and the value it had after it. A row the edit
//! left alone has no entry at all, so a record is proportional to the edit and never to the Space:
//! renaming one card costs the same in a Space of ten thousand cards as in a Space of ten. Replay
//! writes one side of those rows back and revalidates the Space it produced.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use super::RelationLifecycle;
use crate::spaces::structure::{repository::rows::StructureRow, RelationColor, StructureLayout};

mod state;

pub(crate) use state::{capture_state, HistoryState};

/// One row: `None` means the row was absent from the Space on that side of the edit.
#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Change<T> {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<T>,
}

/// A node's own row belongs to the node tables and an edit never rewrites it; only its presence in
/// this Space changes, and that is what a replay has to put back.
#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Presence {
    pub id: String,
    pub before: bool,
    pub after: bool,
}

#[derive(Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct HistoryPatch {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub structure: Vec<Change<StructureRow>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub relation_styles: Vec<Change<RelationColor>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub nodes: Vec<Presence>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub relations: Vec<Change<RelationLifecycle>>,
}

#[derive(Clone, Copy)]
pub(super) enum Side {
    Before,
    After,
}

impl Side {
    pub fn of(redo: bool) -> Self {
        if redo {
            Self::After
        } else {
            Self::Before
        }
    }

    pub fn row<T>(self, change: &Change<T>) -> Option<&T> {
        match self {
            Self::Before => change.before.as_ref(),
            Self::After => change.after.as_ref(),
        }
    }

    fn present(self, presence: &Presence) -> bool {
        match self {
            Self::Before => presence.before,
            Self::After => presence.after,
        }
    }
}

pub(super) fn diff(before: &HistoryState, after: &HistoryState) -> HistoryPatch {
    HistoryPatch {
        structure: changes(&before.structure, &after.structure),
        relation_styles: changes(&before.relation_styles, &after.relation_styles),
        nodes: presences(&before.live_nodes(), &after.live_nodes()),
        relations: changes(&before.relations, &after.relations),
    }
}

impl HistoryPatch {
    /// Structure rows this record can write or remove, which is also the set a replay must take out
    /// of the live order before it writes any of them back.
    pub fn structure_ids(&self) -> Vec<String> {
        self.structure
            .iter()
            .map(|change| change.id.clone())
            .collect()
    }

    /// Structure rows absent on this side: the record restores the Space to a state without them.
    pub fn absent_structure(&self, side: Side) -> Vec<String> {
        self.structure
            .iter()
            .filter(|change| side.row(change).is_none())
            .map(|change| change.id.clone())
            .collect()
    }

    /// Containers this record puts back into the Space; a Portal can target one of them.
    pub fn containers(&self, side: Side) -> Vec<&str> {
        self.structure
            .iter()
            .filter(|change| {
                side.row(change)
                    .is_some_and(|row| row.node.kind_name() != "content")
            })
            .map(|change| change.id.as_str())
            .collect()
    }

    /// Nodes this side expects to find in this Space.
    pub fn present_nodes(&self, side: Side) -> Vec<String> {
        self.nodes
            .iter()
            .filter(|presence| side.present(presence))
            .map(|presence| presence.id.clone())
            .collect()
    }

    /// Every node whose presence this record changed: the only nodes whose derived state, Portals
    /// or dependent rows a replay has to revisit.
    pub fn touched_nodes(&self) -> Vec<String> {
        self.nodes
            .iter()
            .map(|presence| presence.id.clone())
            .collect()
    }
}

fn changes<T: Clone + PartialEq>(
    before: &BTreeMap<String, T>,
    after: &BTreeMap<String, T>,
) -> Vec<Change<T>> {
    before
        .keys()
        .chain(after.keys())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .filter(|id| before.get(*id) != after.get(*id))
        .map(|id| Change {
            id: id.clone(),
            before: before.get(id).cloned(),
            after: after.get(id).cloned(),
        })
        .collect()
}

fn presences(before: &BTreeSet<String>, after: &BTreeSet<String>) -> Vec<Presence> {
    before
        .symmetric_difference(after)
        .map(|id| Presence {
            id: id.clone(),
            before: before.contains(id),
            after: after.contains(id),
        })
        .collect()
}

/// The layout a replay writes for one structure row: the stored side, except where the edit left a
/// content size alone. Sizes also change outside history — fitting a card to the media it shows —
/// so undoing an unrelated move must not drag a stale size back with it.
pub(super) fn layout_to_write(
    change: &Change<StructureRow>,
    side: Side,
    current: Option<&StructureLayout>,
) -> Option<StructureLayout> {
    let mut layout = side.row(change)?.layout.clone();
    let (Some(before), Some(after), Some(current)) = (
        change.before.as_ref().and_then(content_size),
        change.after.as_ref().and_then(content_size),
        current.and_then(layout_content_size),
    ) else {
        return Some(layout);
    };
    if let StructureLayout::Content {
        size, sizing_mode, ..
    } = &mut layout
    {
        if before.0 == after.0 {
            size.width = current.0;
        }
        if before.1 == after.1 {
            size.height = current.1;
        }
        if before.2 == after.2 {
            *sizing_mode = current.2;
        }
    }
    Some(layout)
}

type ContentSize = (f64, f64, crate::spaces::structure::SizingMode);

fn content_size(row: &StructureRow) -> Option<ContentSize> {
    layout_content_size(&row.layout)
}

fn layout_content_size(layout: &StructureLayout) -> Option<ContentSize> {
    match layout {
        StructureLayout::Content {
            size, sizing_mode, ..
        } => Some((size.width, size.height, *sizing_mode)),
        _ => None,
    }
}

#[cfg(test)]
mod tests;

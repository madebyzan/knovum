use std::collections::HashSet;

use super::{Boundary, Crossing, RELATION_SPLIT};

fn ends(ids: &[&str]) -> Vec<String> {
    ids.iter().map(|id| (*id).to_owned()).collect()
}

fn crossing(moving: &[&str], trashed: &[&str], dependency: &[&str]) -> Result<Crossing, String> {
    let moving: HashSet<&str> = moving.iter().copied().collect();
    let boundary = Boundary {
        moving: &moving,
        trashed: trashed.iter().map(|id| (*id).to_owned()).collect(),
    };
    boundary.crossing(&ends(dependency), RELATION_SPLIT)
}

#[test]
fn a_dependency_moves_only_with_all_its_ends() {
    assert_eq!(
        crossing(&["sheet", "sticker"], &[], &["sticker", "sheet"]),
        Ok(Crossing::Moves)
    );
    assert_eq!(
        crossing(&["other"], &[], &["sticker", "sheet"]),
        Ok(Crossing::Stays)
    );
}

#[test]
fn a_dependency_on_a_live_node_left_behind_refuses_the_move() {
    assert_eq!(
        crossing(&["sheet"], &[], &["sticker", "sheet"]),
        Err(RELATION_SPLIT.to_owned())
    );
}

#[test]
fn a_dependency_on_trashed_content_is_severed() {
    // A sticker deleted from an earlier slicing still has its relation to the sheet, kept so undo
    // can bring it back; it does not hold the sheet and its live stickers in the source Space.
    assert_eq!(
        crossing(
            &["sheet"],
            &["deleted-sticker"],
            &["deleted-sticker", "sheet"]
        ),
        Ok(Crossing::Severs)
    );
}

#[test]
fn trashed_content_does_not_excuse_a_live_end_left_behind() {
    // Provenance on a relation ties three nodes: one trashed end cannot make up for a live one.
    assert_eq!(
        crossing(&["source"], &["trashed"], &["source", "trashed", "live"]),
        Err(RELATION_SPLIT.to_owned())
    );
}

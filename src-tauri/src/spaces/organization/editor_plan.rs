use super::{context::Context, draft::Draft, editor_command::EditorCommand, frame_edit, appearance_edit, layer_order};
use crate::spaces::canvas_state::{self, contract::{SpaceEditorDocument, CanvasRelationStyleV1}};
pub(in crate::spaces) fn plan_editor(before: &SpaceEditorDocument, command: &EditorCommand) -> Result<super::Plan, String> {
    match command {
        EditorCommand::Structure { command, selection } => return super::plan(before, command, selection),
        EditorCommand::DeleteSelection { ids, .. } => {
            if ids.is_empty() { return Ok(super::Plan { after: before.clone(), selection: vec![], removed_content_ids: vec![] }); }
            return super::plan(before, &super::StructureCommand::DeleteSubtrees { ids: ids.clone() }, &[]);
        },
        EditorCommand::MeasureText { .. } => return Err("Text measurements require the authoritative content read set".into()),
        _ => {},
    }
    let context = Context::new(before)?; let mut draft = Draft::new(before);
    let selection = match command {
        EditorCommand::TransformFrames { updates } => frame_edit::transform(&context, &mut draft, updates)?,
        EditorCommand::ArrangeFrames { updates } => frame_edit::arrange(&context, &mut draft, updates)?,
        EditorCommand::SetAppearance { item_id, changes } => { appearance_edit::apply(&context, &mut draft, item_id, changes)?; vec![item_id.clone()] },
        EditorCommand::MoveLayer { ids, mode } => layer_order::apply(&context, &mut draft, ids, *mode)?,
        EditorCommand::SetRelationColor { .. } => vec![],
        _ => return Err("Unsupported editor intent".into()),
    };
    let mut after = draft.finish(before);
    if let EditorCommand::SetRelationColor { relation_id, color } = command {
        let mut styles: Vec<_> = after.relation_styles.take().unwrap_or_default().into_iter().filter(|style| style.relation_id != *relation_id).collect();
        if let Some(color) = color { styles.push(CanvasRelationStyleV1 { relation_id: relation_id.clone(), color: color.clone() }); }
        styles.sort_by(|a,b| a.relation_id.cmp(&b.relation_id));
        after.relation_styles = if styles.is_empty() { None } else { Some(styles) };
    }
    canvas_state::validate_document(&after)?;
    Ok(super::Plan { after, selection, removed_content_ids: vec![] })
}

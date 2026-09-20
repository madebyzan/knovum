use serde::{Deserialize, Serialize};
use super::contract::{SpaceEditorDocument, SpaceStructure, StructureLayoutDocument, CanvasRelationStyleV1, CanvasViewport};
/** The camera is assembled at read boundaries and is never serialized into edit storage. */
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct StoredEditorDocument {
    version: u8,
    structure: SpaceStructure,
    layout: StructureLayoutDocument,
    #[serde(skip_serializing_if = "Option::is_none")] relation_styles: Option<Vec<CanvasRelationStyleV1>>,
}
impl StoredEditorDocument {
    pub fn from_document(document: &SpaceEditorDocument) -> Self {
        Self { version: document.version, structure: document.structure.clone(), layout: document.layout.clone(), relation_styles: document.relation_styles.clone() }
    }
    pub fn into_document(self, viewport: CanvasViewport) -> SpaceEditorDocument {
        SpaceEditorDocument { version: self.version, structure: self.structure, layout: self.layout, relation_styles: self.relation_styles, viewport }
    }
}

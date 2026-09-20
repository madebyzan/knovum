"""Reviewed, one-time canonical document cutover on PR 14; not a runtime migration."""
from pathlib import Path
import re

ROOT = Path('.')
updates = {}

def read(name):
    return updates.get(name, (ROOT / name).read_text())

def write(name, value):
    updates[name] = value.strip() + '\n'

# Rename the persistent document identity consistently. No alias or legacy reader survives.
for base in ('src', 'tests', 'src-tauri/src'):
    for path in (ROOT / base).rglob('*'):
        if path.suffix not in ('.ts', '.vue', '.rs'):
            continue
        source = path.read_text()
        after = re.sub(r'\bCanvasDocumentV1\b', 'SpaceEditorDocument', source)
        after = re.sub(r'\bparseCanvasDocument\b', 'parseSpaceEditorDocument', after)
        if after != source:
            updates[str(path)] = after

appearance_path = 'src/features/spaces/canvas/contract/canvas-appearance.ts'
if not (ROOT / appearance_path).exists():
    source = read('src/features/spaces/canvas/contract/canvas-document.ts')
    start = source.index('export const CANVAS_TYPOGRAPHY_SCALE_MIN')
    end = source.index('const canvasNodeSizeAuthoritySchema')
    appearance = source[start:end].replace('const finiteNumber = z.number().finite();\n', '')
    write(appearance_path, 'import { z } from "zod";\n\n' + appearance + '\nexport type CanvasNodeAppearance = z.infer<typeof canvasNodeAppearanceSchema>;\n')

write('src/features/spaces/canvas/contract/canvas-document.ts', '''
import { isEqual } from "es-toolkit";
import { z } from "zod";
import { spaceStructureSchema } from "../../model/structure/structure-schema";
import { validateStructureSnapshot } from "../../organization/structure/structure-snapshot-validation";
import { canvasNodeAppearanceSchema } from "./canvas-appearance";
import { structureLayoutSchema } from "./structure-layout";
export { CANVAS_FILL_PRESETS, CANVAS_TEXT_COLORS, CANVAS_TYPOGRAPHY_SCALE_MIN, CANVAS_TYPOGRAPHY_SCALE_MAX, canvasNodeAppearanceSchema } from "./canvas-appearance";
export type { CanvasNodeAppearance } from "./canvas-appearance";
import type { CanvasNodeAppearance } from "./canvas-appearance";

export const MAX_CANVAS_Z_INDEX = 0xffff_ffff;
export const CANVAS_RELATION_COLORS = ["neutral", "blue", "green", "amber", "red", "purple"] as const;
export const CANVAS_RELATION_OVERRIDE_COLORS = ["blue", "green", "amber", "red", "purple"] as const;
export const canvasViewportSchema = z.object({ x: z.number().finite(), y: z.number().finite(), zoom: z.number().finite().positive() }).strict();
export const canvasRelationStyleSchema = z.object({ relationId: z.string().min(1).max(128), color: z.enum(CANVAS_RELATION_OVERRIDE_COLORS) }).strict();
export const spaceEditorDocumentSchema = z.object({
  version: z.literal(1), structure: spaceStructureSchema, layout: structureLayoutSchema,
  viewport: canvasViewportSchema,
  relationStyles: z.array(canvasRelationStyleSchema).max(100_000).optional(),
}).strict().superRefine((document, context) => {
  try { validateStructureSnapshot(document); }
  catch (error) { context.addIssue({ code: "custom", message: String(error), path: ["structure"] }); }
  const identities = new Set<string>();
  for (const [index, style] of (document.relationStyles ?? []).entries()) {
    if (identities.has(style.relationId)) context.addIssue({ code: "custom", message: "Duplicate relation presentation identity", path: ["relationStyles", index] });
    identities.add(style.relationId);
  }
});
export type SpaceEditorDocument = z.infer<typeof spaceEditorDocumentSchema>;
export type CanvasRelationColor = (typeof CANVAS_RELATION_COLORS)[number];
export type CanvasRelationOverrideColor = (typeof CANVAS_RELATION_OVERRIDE_COLORS)[number];
export type CanvasNodeSizeAuthority = "content" | "frame";
export interface CanvasNodeFrame { x: number; y: number; width: number; height: number; rotation: number }
/** Disposable world-space projection. This record is never persisted as organization state. */
export interface CanvasNodeLayoutV1 {
  nodeId: string; frame: CanvasNodeFrame; sizeAuthority: CanvasNodeSizeAuthority; zIndex: number;
  contentScale?: number; appearance?: CanvasNodeAppearance; fileSubpath?: string;
}
export interface CanvasNodeFrameUpdate { nodeId: string; frame: CanvasNodeFrame; sizeAuthority?: CanvasNodeSizeAuthority }
export function parseSpaceEditorDocument(value: unknown): SpaceEditorDocument { return spaceEditorDocumentSchema.parse(value); }
export function emptySpaceEditorDocument(): SpaceEditorDocument {
  return { version: 1, structure: { version: 1, items: [] }, layout: { version: 1, items: [] }, viewport: { x: 0, y: 0, zoom: 1 } };
}
export function sameCanvasNodeAppearance(left: CanvasNodeAppearance | undefined, right: CanvasNodeAppearance | undefined) { return isEqual(left, right); }
''')
source = read('src/features/spaces/canvas/contract/structure-layout.ts')
write('src/features/spaces/canvas/contract/structure-layout.ts', source.replace('from "./canvas-document"', 'from "./canvas-appearance"'))
# Strict package ownership imports the current document schema, never the old flat envelope.
for path in (ROOT / 'src').rglob('*.ts'):
    if str(path) == 'src/features/spaces/canvas/contract/canvas-document.ts':
        continue
    text = read(str(path))
    text = re.sub(r'\bcanvasDocumentSchema\b', 'spaceEditorDocumentSchema', text)
    write(str(path), text) if text != read(str(path)) else None

write('src/features/spaces/canvas/document/editor-layout-projection.ts', '''
import type { CanvasNodeLayoutV1, SpaceEditorDocument } from "../contract/canvas-document";
import { indexStructureSnapshot } from "../../organization/structure/structure-context";
import { projectStructure, structureBoundsInParent } from "../../organization/structure/structure-projection";
import { transformStructurePoint } from "./structure-transform";
export const SECTION_COLLAPSED_SIZE = { width: 320, height: 64 } as const;

/** World frames, UI order and scale are one-way projections of local geometry and sibling order. */
export function projectEditorLayouts(document: SpaceEditorDocument, expanded = false): CanvasNodeLayoutV1[] {
  const context = indexStructureSnapshot(document);
  const projection = projectStructure(context);
  return [...projection.paintOrder].map(([id, zIndex]) => {
    const layout = context.layouts.get(id)!;
    const world = projection.transforms.get(id)!;
    const bounds = layout.kind === "group"
      ? structureBoundsInParent(context, projection, (context.tree.children.get(id) ?? []).map((item) => item.id), id)!
      : { x: 0, y: 0, ...(layout.kind === "section" && layout.collapsed && !expanded ? SECTION_COLLAPSED_SIZE : layout.size) };
    const point = transformStructurePoint(world, bounds);
    return { nodeId: id, zIndex, contentScale: layout.kind === "content" ? world.scale : 1,
      frame: { ...point, width: bounds.width * world.scale, height: bounds.height * world.scale, rotation: world.rotation },
      sizeAuthority: layout.kind === "content" ? layout.sizeAuthority : "frame",
      ...(layout.kind !== "group" && layout.appearance ? { appearance: layout.appearance } : {}),
      ...(layout.kind === "content" && layout.fileSubpath ? { fileSubpath: layout.fileSubpath } : {}),
    };
  });
}
''')
write('src/features/spaces/canvas/document/canvas-document-sync.ts', '''
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { emptySpaceEditorDocument, parseSpaceEditorDocument, type SpaceEditorDocument } from "../contract/canvas-document";
import { spaceNodeSizePolicy, spaceNodeSupportsContentSizing } from "../node-size";

/** New content use-case only; existing persisted documents are never repaired from renderer nodes. */
export function appendEditorContent(document: SpaceEditorDocument, nodes: readonly SpaceCanvasNode[]): SpaceEditorDocument {
  const first = document.structure.items.filter((item) => item.parentId === null).length;
  return parseSpaceEditorDocument({ ...document,
    structure: { version: 1, items: [...document.structure.items, ...nodes.map((node, index) => ({ kind: "content", id: node.id, nodeId: node.id, parentId: null, order: first + index, locked: false }))] },
    layout: { version: 1, items: [...document.layout.items, ...nodes.map((node) => {
      const { width, height } = spaceNodeSizePolicy(node.data);
      return { kind: "content", itemId: node.id, transform: { ...node.initialPosition, rotation: 0, scale: 1 }, size: { width, height }, sizeAuthority: spaceNodeSupportsContentSizing(node.data) ? "content" : "frame" };
    })] },
  });
}
export function createCanvasDocument(nodes: readonly SpaceCanvasNode[] = []): SpaceEditorDocument { return appendEditorContent(emptySpaceEditorDocument(), nodes); }
''')
write('src/features/spaces/canvas/contract/canvas-display-node.ts', '''
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
export interface CanvasContainerDisplay {
  id: string;
  data: { kind: "group"; title: string; collapsed: false; contentCount: number } | { kind: "section"; title: string; collapsed: boolean; contentCount: number };
}
/** Containers are structural handles, never persistable knowledge content. */
export type CanvasDisplayNode = SpaceCanvasNode | CanvasContainerDisplay;
''')

# Rust persistent contract uses the exact same current structure and layout shape.
contract_path = 'src-tauri/src/spaces/canvas_state/contract.rs'
source = read(contract_path)
appearance_file = 'src-tauri/src/spaces/canvas_state/appearance.rs'
if not (ROOT / appearance_file).exists():
    start = source.index('#[derive(Debug, Deserialize, Serialize)]\n#[serde(rename_all = "lowercase")]\npub(crate) enum CanvasNodeSizeAuthority')
    end = source.index('#[derive(Debug, Deserialize, Serialize)]\n#[serde(deny_unknown_fields)]\npub(crate) struct CanvasNodeFrame')
    write(appearance_file, 'use serde::{Deserialize, Serialize};\n\n' + source[start:end].replace('Debug, Deserialize, Serialize', 'Clone, Debug, PartialEq, Deserialize, Serialize'))
write(contract_path, '''
use serde::{Deserialize, Serialize};
pub(crate) use super::appearance::*;
pub(crate) use super::structure_contract::*;
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct SpaceEditorDocument {
    pub version: u8, pub structure: SpaceStructure, pub layout: StructureLayoutDocument,
    pub viewport: CanvasViewport,
    #[serde(skip_serializing_if = "Option::is_none")] pub relation_styles: Option<Vec<CanvasRelationStyleV1>>,
}
impl SpaceEditorDocument {
    pub(crate) fn empty() -> Self { Self { version: 1, structure: SpaceStructure { version: 1, items: vec![] }, layout: StructureLayoutDocument { version: 1, items: vec![] }, viewport: CanvasViewport { x: 0.0, y: 0.0, zoom: 1.0 }, relation_styles: None } }
}
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct CanvasRelationStyleV1 { pub relation_id: String, pub color: CanvasRelationColor }
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum CanvasRelationColor { Blue, Green, Amber, Red, Purple }
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CanvasViewport { pub x: f64, pub y: f64, pub zoom: f64 }
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CanvasNodeFrame { pub x: f64, pub y: f64, pub width: f64, pub height: f64, pub rotation: f64 }
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StoredCanvasState { pub document: SpaceEditorDocument, pub revision: i64 }
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct SaveCanvasStateRequest { pub space_id: String, pub base_revision: i64, pub document: SpaceEditorDocument }
''')
write('src-tauri/src/spaces/canvas_state/structure_contract.rs', '''
use serde::{Deserialize, Serialize};
use super::appearance::{CanvasNodeAppearance, CanvasNodeSizeAuthority};
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum StructureKind { Content, Group, Section }
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct StructureItem {
    pub id: String, pub kind: StructureKind, pub parent_id: Option<String>, pub order: u32, pub locked: bool,
    #[serde(skip_serializing_if = "Option::is_none")] pub node_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")] pub title: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SpaceStructure { pub version: u8, pub items: Vec<StructureItem> }
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct StructureLayoutDocument { pub version: u8, pub items: Vec<StructureLayout> }
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "lowercase", deny_unknown_fields)]
pub(crate) enum StructureLayout {
    Content { #[serde(rename = "itemId")] item_id: String, transform: StructureTransform, size: StructureSize,
        #[serde(rename = "sizeAuthority")] size_authority: CanvasNodeSizeAuthority,
        #[serde(skip_serializing_if = "Option::is_none")] appearance: Option<CanvasNodeAppearance>,
        #[serde(rename = "fileSubpath", skip_serializing_if = "Option::is_none")] file_subpath: Option<String> },
    Group { #[serde(rename = "itemId")] item_id: String, transform: StructureTransform },
    Section { #[serde(rename = "itemId")] item_id: String, position: StructurePoint, size: StructureSize, collapsed: bool,
        #[serde(skip_serializing_if = "Option::is_none")] appearance: Option<CanvasNodeAppearance> },
}
impl StructureLayout {
    pub(crate) fn id(&self) -> &str { match self { Self::Content { item_id, .. } | Self::Group { item_id, .. } | Self::Section { item_id, .. } => item_id } }
    pub(crate) fn kind(&self) -> StructureKind { match self { Self::Content { .. } => StructureKind::Content, Self::Group { .. } => StructureKind::Group, Self::Section { .. } => StructureKind::Section } }
    pub(crate) fn local(&self) -> StructureTransform { match self { Self::Content { transform, .. } | Self::Group { transform, .. } => *transform, Self::Section { position, .. } => StructureTransform { x: position.x, y: position.y, rotation: 0.0, scale: 1.0 } } }
    pub(crate) fn appearance(&self) -> Option<&CanvasNodeAppearance> { match self { Self::Content { appearance, .. } | Self::Section { appearance, .. } => appearance.as_ref(), Self::Group { .. } => None } }
}
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct StructureTransform { pub x: f64, pub y: f64, pub rotation: f64, pub scale: f64 }
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct StructureSize { pub width: f64, pub height: f64 }
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct StructurePoint { pub x: f64, pub y: f64 }
''')
module = 'src-tauri/src/spaces/canvas_state/mod.rs'
source = read(module)
source = source.replace('mod contract;', 'pub(in crate::spaces) mod contract;\nmod appearance;\nmod structure_contract;')
write(module, source)
# The new runtime rejects the obsolete flat schema; tests and consumers are updated in the following cutover steps.
for name, content in updates.items():
    path = ROOT / name
    path.parent.mkdir(parents=True, exist_ok=True)
    if not path.exists() or path.read_text() != content:
        path.write_text(content)
print(f'Canonical contract cutover reviewed {len(updates)} files.')

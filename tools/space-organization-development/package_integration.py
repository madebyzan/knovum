"""Apply the strict native-package and editor boundary integration as one source edit."""
from pathlib import Path

root = Path.cwd()
marker = root / "tools/space-organization-development/applied/package-integration"
if marker.exists():
    raise SystemExit(0)
sources = {}


def read(path):
    return sources.get(path, (root / path).read_text() if (root / path).exists() else "")


def write(path, text):
    sources[path] = text.strip() + "\n"


def replace(path, before, after):
    source = read(path)
    if before not in source:
        raise RuntimeError(f"Reviewed source precondition failed: {path}: {before[:100]}")
    write(path, source.replace(before, after))


write("src/features/spaces/canvas/contract/editor-result.ts", r'''
import { z } from "zod";

import { SPACE_RELATION_KINDS, SPACE_RELATION_STATUSES } from "../../model/relations/relation";
import { spaceNodePayloadSchema } from "../../model/nodes/schema/node-variants";
import { spaceEditorDocumentSchema } from "./canvas-document";

const identity = z.string().min(1).max(128);
const revision = z.number().int().nonnegative().max(Number.MAX_SAFE_INTEGER);
const relationSchema = z.object({
  id: identity, spaceId: identity, sourceId: identity, targetId: identity,
  kind: z.enum(SPACE_RELATION_KINDS), rationale: z.string(), status: z.enum(SPACE_RELATION_STATUSES),
  statusBeforeInvalidation: z.enum(["active", "contested"]).nullable(),
  confidence: z.number().finite().min(0).max(1).nullable(),
  actorKind: z.enum(["human", "assistant", "agent", "system"]), actorId: identity.nullable(),
  createdAt: z.number().int(), updatedAt: z.number().int(),
}).strict().transform(({ confidence, actorId, ...relation }) => ({
  ...relation,
  ...(confidence === null ? {} : { confidence }),
  ...(actorId === null ? {} : { actorId }),
}));
export const editorContentSchema = z.object({
  nodes: z.array(z.object({
    id: identity, initialX: z.number().finite(), initialY: z.number().finite(), data: spaceNodePayloadSchema,
  }).strict()),
  relations: z.array(relationSchema),
}).strict();
export const spaceEditResultSchema = z.object({
  document: spaceEditorDocumentSchema, revision, appliedAtRevision: revision,
  mutationId: identity, selection: z.array(identity), historyReceiptId: identity.nullable(),
  content: editorContentSchema.nullable(),
}).strict().refine((result) => result.appliedAtRevision <= result.revision, "A receipt cannot be ahead of the current editor state");
export const editorWorkspaceSchema = z.object({
  document: spaceEditorDocumentSchema, revision, content: editorContentSchema,
}).strict();
export type SpaceEditResult = z.infer<typeof spaceEditResultSchema>;
export type EditorWorkspace = z.infer<typeof editorWorkspaceSchema>;
export type EditorWorkspaceContent = z.infer<typeof editorContentSchema>;
''')
write("src/features/spaces/canvas/canvas-package-codec.ts", r'''
import type { CanvasPackageCodec } from "../package/canvas-state-codec";
import { PACKAGE_ASSET_PREFIX } from "../package/contract";
import { parseSpaceEditorDocument } from "./contract/canvas-document";

export const canvasPackageCodec: CanvasPackageCodec = {
  validate: (source) => { parseSpaceEditorDocument(source.document); },
  remap(source, context) {
    const document = parseSpaceEditorDocument(source.document);
    const itemId = (id: string) => mapped(context.itemIds, id, "structure item");
    return { document: parseSpaceEditorDocument({
      ...document,
      structure: { version: 1, items: document.structure.items.map((item) => ({
        ...item, id: itemId(item.id), parentId: item.parentId === null ? null : itemId(item.parentId),
        ...(item.kind === "content" ? { nodeId: mapped(context.nodeIds, item.nodeId, "content") } : {}),
      })) },
      layout: { version: 1, items: document.layout.items.map((layout) => {
        const copy = structuredClone(layout);
        copy.itemId = itemId(layout.itemId);
        if (copy.kind !== "group" && copy.appearance?.background) {
          const path = copy.appearance.background;
          if (!path.startsWith(PACKAGE_ASSET_PREFIX)) throw new Error("Package backgrounds must reference packaged resources");
          copy.appearance.background = mapped(context.assets, path.slice(PACKAGE_ASSET_PREFIX.length), "background asset");
        }
        return copy;
      }) },
      relationStyles: (document.relationStyles ?? []).map((style) => ({
        ...style, relationId: mapped(context.relationIds, style.relationId, "relation"),
      })),
    }) };
  },
};

function mapped(mapping: ReadonlyMap<string, string>, id: string, kind: string) {
  const value = mapping.get(id);
  if (!value) throw new Error(`Package references an unknown ${kind}: ${id}`);
  return value;
}
''')
replace("src/features/spaces/package/canvas-state-codec.ts", "  nodeIds: ReadonlyMap<string, string>;", "  nodeIds: ReadonlyMap<string, string>;\n  itemIds: ReadonlyMap<string, string>;\n  relationIds: ReadonlyMap<string, string>;\n  assets: ReadonlyMap<string, string>;")
replace("src/features/spaces/package/id-maps.ts", "  nodes: Map<string, string>;", "  nodes: Map<string, string>;\n  items: Map<string, string>;")
replace("src/features/spaces/package/id-maps.ts", "  return {\n    sourceSpaceId:", "  const nodes = idMap(manifest.nodes);\n  const items = new Map(manifest.canvasState.document.structure.items.map((item) => [item.id, item.kind === \"content\" ? requiredMap(nodes, item.nodeId, \"content\") : crypto.randomUUID()]));\n  return {\n    sourceSpaceId:")
replace("src/features/spaces/package/id-maps.ts", "    nodes: idMap(manifest.nodes),", "    nodes,\n    items,")
replace("src/features/spaces/package/id-maps.ts", "map: Map<string, string>", "map: ReadonlyMap<string, string>")
replace("src/features/spaces/package/restore-request.ts", "      nodeIds: context.maps.nodes,", "      nodeIds: context.maps.nodes,\n      itemIds: context.maps.items,\n      relationIds: context.maps.relations,\n      assets: context.assets,")
replace("src/features/spaces/package/restore-request.ts", "    title: scene.title,", "    title: scene.title,\n    thumbnailPath: null,")
replace("src/features/spaces/package/restore-request.ts", 'requiredMap(maps.nodes, id, "Scene focus node")', 'requiredMap(maps.items, id, "Scene focus item")')
replace("src/features/spaces/package/content-remap.ts", 'requiredMap(maps.nodes, data.targetNodeId, "portal node")', 'requiredMap(maps.items, data.targetNodeId, "portal item")')
replace("src/features/spaces/package/content-remap.ts", '  if (copy.kind === "portal") remapPortal(copy, maps);', '  if (copy.kind === "portal") remapPortal(copy, maps);\n  if (copy.kind === "subtitle") copy.sourceVideoNodeId = requiredMap(maps.nodes, copy.sourceVideoNodeId, "subtitle source");')
write("src/features/spaces/package/structure-validation.ts", r'''
import type { SpacePackageManifest } from "./manifest-schema";

export function validatePackagedStructure(manifest: SpacePackageManifest) {
  const document = manifest.canvasState.document;
  const contentIds = new Set(document.structure.items.flatMap((item) => item.kind === "content" ? [item.nodeId] : []));
  const nodeIds = new Set(manifest.nodes.map((node) => node.id));
  if (nodeIds.size !== manifest.nodes.length || nodeIds.size !== contentIds.size || [...nodeIds].some((id) => !contentIds.has(id))) {
    throw new Error("Packaged content and structure must match exactly");
  }
  const itemIds = new Set(document.structure.items.map((item) => item.id));
  for (const scene of manifest.scenes) {
    if (scene.focusNodeIds.some((id) => !itemIds.has(id))) throw new Error("Packaged Saved View references an unavailable structure item");
  }
  const relations = new Set(manifest.relations.map((relation) => relation.id));
  if ((document.relationStyles ?? []).some((style) => !relations.has(style.relationId))) {
    throw new Error("Packaged relationship presentation references an unavailable relationship");
  }
}
''')
replace("src/features/spaces/package/ownership-validation.ts", 'import type { SpacePackageManifest }', 'import { validatePackagedStructure } from "./structure-validation";\nimport type { SpacePackageManifest }')
replace("src/features/spaces/package/ownership-validation.ts", "  validateSpaceOwnedRows(manifest);", "  validatePackagedStructure(manifest);\n  validateSpaceOwnedRows(manifest);")
# Read the complete current editor in the same export transaction, then rewrite managed backgrounds.
path = "src-tauri/src/spaces/queries/export/rows.rs"
source = read(path)
start = source.index("pub(super) async fn canvas_state(")
end = source.index("pub(super) async fn scenes(", start)
source = source[:start] + r'''
pub(super) async fn canvas_state(
    tx: &mut Transaction<'_, Sqlite>, space_id: &str, assets: &mut PackageAssets,
) -> Result<Value, String> {
    let (mut document, _) = crate::spaces::canvas_state::load_in_transaction(tx, space_id).await?;
    crate::spaces::canvas_state::validation::validate_content_references(tx, space_id, &document).await?;
    for layout in &mut document.layout.items {
        use crate::spaces::canvas_state::contract::StructureLayout;
        match layout {
            StructureLayout::Content { appearance, .. } | StructureLayout::Section { appearance, .. } => {
                if let Some(path) = appearance.as_mut().and_then(|appearance| appearance.background.as_mut()) {
                    *path = assets.reference(path);
                }
            }
            StructureLayout::Group { .. } => {}
        }
    }
    Ok(serde_json::json!({ "document": document }))
}

''' + source[end:]
write(path, source)
replace("src-tauri/src/spaces/queries/export/mod.rs", "rows::canvas_state(&mut tx, &space_id).await?", "rows::canvas_state(&mut tx, &space_id, &mut assets).await?")
replace("src-tauri/src/spaces/package/manifest_assets/mod.rs", "    Ok(ManifestAssets {", r'''
    if let Some(layouts) = value.pointer("/canvasState/document/layout/items").and_then(Value::as_array) {
        for layout in layouts {
            if let Some(reference) = layout.pointer("/appearance/background").and_then(Value::as_str) {
                let token = package_token(reference)?.ok_or("Packaged backgrounds must reference packaged resources")?;
                used_tokens.insert(token.to_owned());
            }
        }
    }
    Ok(ManifestAssets {''')
# Native restore validates the supplied complete graph, not serialized legacy canvas rows.
path = "src-tauri/src/spaces/package/restore/mod.rs"
source = read(path)
start = source.index("    crate::spaces::canvas_state::validate_document", source.index("async fn insert_canvas_state("))
end = source.index("\n    Ok(())", start)
source = source[:start] + r'''
    crate::spaces::canvas_state::save_in_transaction(tx, &request.space.id, 0, &request.canvas_state.document).await?;
    crate::spaces::canvas_state::viewport::save_in_transaction(tx, &request.space.id, &request.canvas_state.document.viewport).await?;
''' + source[end:]
source = source.replace("    insert_scenes(&mut tx, &request).await?;", "    insert_scenes(&mut tx, &request).await?;\n    validate_nodes(&mut tx, &request).await?;")
start = source.index("    for row in &request.nodes {\n        knowledge_contracts::validate_node")
source = source[:start] + r'''
    Ok(())
}

async fn validate_nodes(tx: &mut Transaction<'_, Sqlite>, request: &RestorePackageRequest) -> Result<(), String> {
''' + source[start:]
source = source.replace('''        for id in &row.focus_node_ids {
            knowledge_contracts::validate_same_space_node(tx, &request.space.id, "", id).await?;
        }''', '''        let item_ids: std::collections::HashSet<&str> = request.canvas_state.document.structure.items.iter().map(|item| item.id.as_str()).collect();
        let mut focus_ids = std::collections::HashSet::new();
        for id in &row.focus_node_ids {
            if !item_ids.contains(id.as_str()) || !focus_ids.insert(id.as_str()) { return Err("Saved View focus must reference unique active structure items".into()); }
        }''')
write(path, source)
write("src/features/spaces/__tests__/package-structure-remap.test.ts", r'''
import { describe, expect, it } from "vitest";

import { canvasPackageCodec } from "../canvas/canvas-package-codec";
import { parseSpaceEditorDocument } from "../canvas/contract/canvas-document";
import { PACKAGE_ASSET_PREFIX } from "../package/contract";

function source() {
  return { document: parseSpaceEditorDocument({ version: 1, viewport: { x: 25, y: -10, zoom: 0.8 },
    structure: { version: 1, items: [
      { kind: "section", id: "s", title: "References", parentId: null, order: 0, locked: false },
      { kind: "group", id: "g", title: "Pair", parentId: "s", order: 0, locked: false },
      { kind: "content", id: "a", nodeId: "a", parentId: "g", order: 0, locked: true },
      { kind: "content", id: "b", nodeId: "b", parentId: "g", order: 1, locked: false },
    ] },
    layout: { version: 1, items: [
      { kind: "section", itemId: "s", position: { x: 100, y: 200 }, size: { width: 800, height: 600 }, collapsed: true, appearance: { background: `${PACKAGE_ASSET_PREFIX}cover` } },
      { kind: "group", itemId: "g", transform: { x: 20, y: 40, scale: 1.5, rotation: 25 } },
      { kind: "content", itemId: "a", transform: { x: 0, y: 0, scale: 1, rotation: 0 }, size: { width: 120, height: 90 }, sizeAuthority: "frame", fileSubpath: "#page=4" },
      { kind: "content", itemId: "b", transform: { x: 180, y: 0, scale: 1, rotation: 0 }, size: { width: 120, height: 90 }, sizeAuthority: "content" },
    ] }, relationStyles: [{ relationId: "r", color: "blue" }],
  }) };
}
function context() {
  return { itemIds: new Map(["s", "g", "a", "b"].map((id) => [id, `new-${id}`])),
    nodeIds: new Map(["a", "b"].map((id) => [id, `new-${id}`])),
    relationIds: new Map([["r", "new-r"]]), assets: new Map([["cover", "/managed/cover.png"]]) };
}

describe("native package structural round trip", () => {
  it("remaps every structural identity while retaining local geometry and presentation", () => {
    const before = source();
    const after = canvasPackageCodec.remap(before, context());
    expect(after.document.structure.items[1]).toMatchObject({ id: "new-g", parentId: "new-s" });
    expect(after.document.structure.items[2]).toMatchObject({ id: "new-a", nodeId: "new-a", parentId: "new-g", locked: true });
    expect(after.document.layout.items[1]).toEqual({ ...before.document.layout.items[1], itemId: "new-g" });
    expect(after.document.layout.items[2]).toMatchObject({ fileSubpath: "#page=4", sizeAuthority: "frame" });
    expect(after.document.layout.items[0]).toMatchObject({ collapsed: true, appearance: { background: "/managed/cover.png" } });
    expect(after.document.viewport).toEqual(before.document.viewport);
    expect(after.document.relationStyles).toEqual([{ relationId: "new-r", color: "blue" }]);
    expect(before).toEqual(source());
  });
  it("rejects missing container or resource mappings instead of dropping data", () => {
    const missingItem = context(); missingItem.itemIds.delete("g");
    expect(() => canvasPackageCodec.remap(source(), missingItem)).toThrow(/structure item/u);
    const missingAsset = context(); missingAsset.assets.clear();
    expect(() => canvasPackageCodec.remap(source(), missingAsset)).toThrow(/background asset/u);
  });
  it("rejects unmanaged file references in an external package", () => {
    const value = source();
    const section = value.document.layout.items[0]!;
    if (section.kind !== "section") throw new Error("Invalid fixture");
    section.appearance = { background: "/external/private.png" };
    expect(() => canvasPackageCodec.remap(value, context())).toThrow(/packaged resources/u);
  });
});
''')
for path, content in sources.items():
    destination = root / path
    destination.parent.mkdir(parents=True, exist_ok=True)
    destination.write_text(content)
marker.parent.mkdir(parents=True, exist_ok=True)
marker.write_text("Reviewed package integration applied. This development marker is removed before delivery.\n")
print(f"Applied package integration to {len(sources)} source files")

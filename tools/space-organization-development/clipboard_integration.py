"""Integrate complete structural clipboard fragments with native editing receipts."""
from pathlib import Path
import re

root = Path.cwd()
marker = root / "tools/space-organization-development/applied/clipboard-integration"
if marker.exists():
    raise SystemExit(0)
sources = {}


def read(path):
    return sources[path] if path in sources else (root / path).read_text()


def write(path, text):
    sources[path] = text.strip() + "\n"


def replace(path, before, after):
    source = read(path)
    if before not in source:
        raise RuntimeError(f"Reviewed source precondition failed: {path}: {before[:100]}")
    write(path, source.replace(before, after))


# A knowledge field schema belongs to its domain, not to one exchange format.
path = "src/features/spaces/package/knowledge-contract-schema.ts"
source = read(path)
start = source.index("const fieldSchema =")
end = source.index("export const objectTypeSchema", start)
field = source[start:end].replace("const fieldSchema =", "export const objectFieldSchema =")
write("src/features/spaces/knowledge/model/object-field-schema.ts", 'import { z } from "zod";\n\n' + field)
source = source[:start] + source[end:]
source = source.replace('import { z } from "zod";', 'import { z } from "zod";\nimport { objectFieldSchema } from "../knowledge/model/object-field-schema";').replace("z.array(fieldSchema)", "z.array(objectFieldSchema)")
write(path, source)
write("src/features/spaces/canvas/contract/editor-fragment.ts", r'''
import type { SpaceStructure } from "../../model/structure/structure-schema";
import type { SpaceNodePayload } from "../../model/nodes/node-payload";
import type { SpaceRelation } from "../../model/relations/relation";
import type { KnowledgeChunkSeed } from "../../knowledge/model/schema";
import type { SpaceKnowledgeObjectType } from "../../knowledge/model/object-types";
import type { StructureLayout } from "./structure-layout";
import type { CanvasRelationOverrideColor } from "./canvas-document";

export interface EditorFragment {
  structure: SpaceStructure;
  layout: StructureLayout;
  nodes: { id: string; kind: SpaceNodePayload["kind"]; data: SpaceNodePayload; sourceChunks: readonly KnowledgeChunkSeed[] }[];
  relations: { id: string; sourceId: string; targetId: string; kind: SpaceRelation["kind"]; rationale: string; status: "active" | "contested"; confidence?: number }[];
  objectTypes: Pick<SpaceKnowledgeObjectType, "id" | "name" | "icon" | "description" | "fields">[];
  relationStyles: { relationId: string; color: CanvasRelationOverrideColor }[];
}
export interface FragmentPlacement { parentId: string | null; delta: { x: number; y: number } }
''')
replace("src/features/spaces/canvas/contract/editor-command.ts", 'import type { StructureCommand }', 'import type { EditorFragment, FragmentPlacement } from "./editor-fragment";\nimport type { StructureCommand }')
replace("src/features/spaces/canvas/contract/editor-command.ts", '  | { kind: "structure";', '  | { kind: "insert-fragment"; fragment: EditorFragment; placement: FragmentPlacement }\n  | { kind: "structure";')
replace("src/features/spaces/canvas/contract/canvas-editor-port.ts", 'import type { StructureCommand }', 'import type { EditorFragment, FragmentPlacement } from "./editor-fragment";\nimport type { StructureCommand }')
replace("src/features/spaces/canvas/contract/canvas-editor-port.ts", '  cancelInteraction(): void;', '  cancelInteraction(): void;\n  insertFragment(fragment: EditorFragment, placement: FragmentPlacement): Promise<void>;')
replace("src/features/spaces/canvas/konva/create-canvas-editor-port.ts", '    cancelInteraction: options.cancelInteraction,', '''    cancelInteraction: options.cancelInteraction,
    insertFragment: async (fragment, placement) => {
      const result = await options.model.executeNative({ kind: "insert-fragment", fragment, placement });
      options.setSelection(result.selection);
    },''')
write("src/features/spaces/clipboard/payload-types.ts", r'''
import type { z } from "zod";
import type { spaceClipboardPayloadSchema } from "./payload-schema";
export type SpaceClipboardPayload = z.infer<typeof spaceClipboardPayloadSchema>;
export type SpaceClipboardNode = SpaceClipboardPayload["nodes"][number];
''')
write("src/features/spaces/clipboard/payload-schema.ts", r'''
import { z } from "zod";
import { spaceStructureSchema } from "../model/structure/structure-schema";
import { structureLayoutSchema } from "../canvas/contract/structure-layout";
import { spaceNodePayloadSchema } from "../model/nodes/schema/node-variants";
import { SPACE_RELATION_KINDS } from "../model/relations/relation";
import { CANVAS_RELATION_OVERRIDE_COLORS } from "../canvas/contract/canvas-document";
import { objectFieldSchema } from "../knowledge/model/object-field-schema";
import { validateStructureSnapshot } from "../organization/structure/structure-snapshot-validation";

const id = z.string().min(1).max(128);
export const spaceClipboardPayloadSchema = z.object({
  version: z.literal(1), kind: z.literal("space-selection"), sourceSpaceId: id,
  structure: spaceStructureSchema, layout: structureLayoutSchema,
  nodes: z.array(z.object({ id, data: spaceNodePayloadSchema }).strict()).max(10_000),
  objectTypes: z.array(z.object({ id, name: z.string().trim().min(1).max(128), icon: id,
    description: z.string().max(4096), fields: z.array(objectFieldSchema).max(64),
  }).strict()).max(10_000),
  relations: z.array(z.object({ id, sourceId: id, targetId: id, kind: z.enum(SPACE_RELATION_KINDS),
    rationale: z.string().max(16_000), status: z.enum(["active", "contested"]),
    confidence: z.number().finite().min(0).max(1).optional(),
    color: z.enum(CANVAS_RELATION_OVERRIDE_COLORS).optional(),
  }).strict()).max(100_000),
}).strict();

export function parseSpaceClipboardPayload(value: unknown) {
  const payload = spaceClipboardPayloadSchema.parse(value);
  if (!payload.structure.items.length || payload.structure.items.length > 10_000) throw new Error("A clipboard selection must contain 1 to 10,000 structural items");
  validateStructureSnapshot(payload);
  const placed = new Set(payload.structure.items.flatMap((item) => item.kind === "content" ? [item.nodeId] : []));
  const nodes = new Set(payload.nodes.map((node) => node.id));
  if (placed.size !== nodes.size || nodes.size !== payload.nodes.length || [...placed].some((id) => !nodes.has(id))) throw new Error("Clipboard content must match its structure exactly");
  const relations = new Set<string>();
  for (const relation of payload.relations) {
    if (!nodes.has(relation.sourceId) || !nodes.has(relation.targetId) || relations.has(relation.id)) throw new Error("Clipboard relationships must be unique and internal to the selection");
    relations.add(relation.id);
  }
  const types = new Set(payload.objectTypes.map((type) => type.id));
  if (types.size !== payload.objectTypes.length || [...types].some((id) => id.startsWith("system:"))) throw new Error("Clipboard custom types must be unique and non-system");
  for (const { data } of payload.nodes) {
    if (data.kind === "object" && !data.objectTypeId.startsWith("system:") && !types.has(data.objectTypeId)) throw new Error("Clipboard is missing a required custom object type");
  }
  return payload;
}
''')
write("src/features/spaces/clipboard/payload.ts", r'''
import type { CanvasSelectionTransfer } from "../canvas/contract/canvas-editor";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceRelation } from "../model/relations/relation";
import type { SpaceKnowledgeObjectType } from "../knowledge/model/object-types";
import { structureSubtree } from "../model/structure/structure-index";
import { structureOperationRoots } from "../organization/structure/structure-selection";
import { indexStructureSnapshot } from "../organization/structure/structure-context";
import { projectStructure } from "../organization/structure/structure-projection";
import { parseSpaceClipboardPayload } from "./payload-schema";

/** A detached native fragment includes hidden descendants and full local geometry. */
export function createSpaceClipboardPayload(input: {
  spaceId: string; transfer: CanvasSelectionTransfer; nodes: readonly SpaceCanvasNode[];
  relations: readonly SpaceRelation[]; objectTypes: readonly SpaceKnowledgeObjectType[];
}) {
  const { document, itemIds } = input.transfer;
  const context = indexStructureSnapshot(document); const geometry = projectStructure(context);
  const roots = structureOperationRoots(context.tree, itemIds).toSorted((a, b) => geometry.paintOrder.get(a.id)! - geometry.paintOrder.get(b.id)!);
  const order = new Map(roots.map((item, index) => [item.id, index]));
  const selected = structureSubtree(context.tree, roots.map((item) => item.id));
  const ids = new Set(selected.flatMap((item) => item.kind === "content" ? [item.nodeId] : []));
  const nodes = input.nodes.filter((node) => ids.has(node.id)).map(({ id, data }) => ({ id, data }));
  const types = new Set(nodes.flatMap(({ data }) => data.kind === "object" ? [data.objectTypeId] : []));
  const colors = new Map(document.relationStyles?.map((style) => [style.relationId, style.color]));
  return parseSpaceClipboardPayload({
    version: 1, kind: "space-selection", sourceSpaceId: input.spaceId,
    structure: { version: 1, items: selected.map((item) => order.has(item.id) ? { ...item, parentId: null, order: order.get(item.id)! } : item) },
    layout: { version: 1, items: selected.map((item) => {
      const layout = context.layouts.get(item.id)!;
      if (!order.has(item.id) || layout.kind === "section") return layout;
      return { ...layout, transform: geometry.transforms.get(item.id)! };
    }) }, nodes,
    objectTypes: input.objectTypes.filter((type) => !type.system && types.has(type.id)).map(({ id, name, icon, description, fields }) => ({ id, name, icon, description, fields })),
    relations: input.relations.filter((relation) => relation.status !== "invalidated" && ids.has(relation.sourceId) && ids.has(relation.targetId)).map((relation) => ({
      id: relation.id, sourceId: relation.sourceId, targetId: relation.targetId, kind: relation.kind,
      rationale: relation.rationale, status: relation.status,
      ...(relation.confidence !== undefined ? { confidence: relation.confidence } : {}),
      ...(colors.has(relation.id) ? { color: colors.get(relation.id) } : {}),
    })),
  });
}
''')
write("src/features/spaces/clipboard/transfer.ts", r'''
import { writeHtmlAndText } from "tauri-plugin-clipboard-api";
import { parseSpaceClipboardPayload } from "./payload-schema";
import type { SpaceClipboardPayload } from "./payload-types";

const MARKER = /<!--knovum-spaces:([^>]*)-->/u;
const MAX_BYTES = 8 * 1024 * 1024;
export async function writeSpaceClipboardDirect(payload: SpaceClipboardPayload) {
  await writeHtmlAndText(clipboardHtml(payload), clipboardText(payload));
}
export function writeSpaceClipboard(event: ClipboardEvent, payload: SpaceClipboardPayload) {
  if (!event.clipboardData) return false;
  event.clipboardData.setData("text/html", clipboardHtml(payload));
  event.clipboardData.setData("text/plain", clipboardText(payload));
  return true;
}
export function readSpaceClipboard(event: ClipboardEvent) { return readSpaceClipboardHtml(event.clipboardData?.getData("text/html") ?? ""); }
export function readSpaceClipboardHtml(html: string): SpaceClipboardPayload | undefined {
  const encoded = html.match(MARKER)?.[1];
  if (encoded === undefined) return undefined;
  if (encoded.length > MAX_BYTES * 3) throw new Error("Native clipboard content is too large");
  const source = decodeURIComponent(encoded);
  if (new TextEncoder().encode(source).byteLength > MAX_BYTES) throw new Error("Native clipboard content is too large");
  // A malformed native payload fails explicitly; it is not silently reinterpreted as plain text.
  return parseSpaceClipboardPayload(JSON.parse(source));
}
function clipboardHtml(value: SpaceClipboardPayload) {
  const source = JSON.stringify(parseSpaceClipboardPayload(value));
  if (new TextEncoder().encode(source).byteLength > MAX_BYTES) throw new Error("Selection exceeds the 8 MiB clipboard limit; export a Space package instead");
  return `<!--knovum-spaces:${encodeURIComponent(source)}-->`;
}
function clipboardText(payload: SpaceClipboardPayload) {
  const parts = payload.nodes.map(({ data }) => {
    if (data.kind === "website") return data.url;
    if ((data.kind === "video" || data.kind === "audio") && data.origin) return data.origin.url;
    if (data.kind === "note" || data.kind === "text") return data.plainText;
    if (data.kind === "markdown") return data.text;
    return data.title;
  }).filter(Boolean);
  return parts.length ? parts.join("\n\n") : payload.structure.items.flatMap((item) => item.kind === "content" ? [] : [item.title]).join("\n");
}
''')
write("src/features/spaces/clipboard/node-cloning.ts", r'''
import { duplicateSpaceAsset } from "../assets/asset-operations";
import { duplicateTiptapAssets } from "../assets/managed-assets";
import { isSpaceManagedAssetPayload, isSpaceWebsitePayload } from "../model/nodes/capabilities";
import type { SpaceNodePayload } from "../model/nodes/node-payload";
import { validateSpaceNodePayload } from "../model/nodes/schema/node-payload";

export interface ContentCloneContext {
  identities: ReadonlyMap<string, string>;
  sourceSpaceId: string;
  targetSpaceId: string;
  assets: string[];
}
export async function cloneNodeData(source: SpaceNodePayload, context: ContentCloneContext) {
  const data = validateSpaceNodePayload(source);
  if (data.kind === "credential") data.credentialConfigured = false;
  if (data.kind === "markdown") delete data.derivationState;
  remapNavigation(data, context);
  if (isSpaceManagedAssetPayload(data)) {
    data.assetPath = await duplicateTracked(data.assetPath, context);
    if ((data.kind === "video" || data.kind === "audio") && data.origin) {
      if (data.previewPath) data.previewPath = await duplicateTracked(data.previewPath, context);
    } else {
      data.processingState = "processing";
      delete data.previewPath;
    }
  }
  if (isSpaceWebsitePayload(data)) {
    if (data.faviconPath) data.faviconPath = await duplicateTracked(data.faviconPath, context);
    if (data.previewImagePath) data.previewImagePath = await duplicateTracked(data.previewImagePath, context);
  }
  await duplicateTiptapAssets(data, (path) => duplicateTracked(path, context));
  return data;
}
function remapNavigation(data: SpaceNodePayload, context: ContentCloneContext) {
  if (data.kind === "portal" && data.targetSpaceId === context.sourceSpaceId && data.targetNodeId && context.identities.has(data.targetNodeId)) {
    if (data.targetSceneId && context.sourceSpaceId !== context.targetSpaceId) throw new Error("A Portal to a Saved View must be copied with a native Space package when changing its target Space");
    data.targetNodeId = context.identities.get(data.targetNodeId)!;
    data.targetSpaceId = context.targetSpaceId;
  }
  if (data.kind === "subtitle") {
    const video = context.identities.get(data.sourceVideoNodeId);
    if (video) data.sourceVideoNodeId = video;
    else if (context.sourceSpaceId !== context.targetSpaceId) throw new Error("Copy the source video together with subtitles when pasting into another Space");
  }
}
async function duplicateTracked(path: string, context: ContentCloneContext) {
  const copy = await duplicateSpaceAsset(path);
  context.assets.push(copy);
  return copy;
}
''')
write("src/features/spaces/clipboard/fragment-types.ts", r'''
import { isEqual } from "es-toolkit";
import type { SpaceKnowledgeObjectType } from "../knowledge/model/object-types";
import type { SpaceClipboardPayload } from "./payload-types";

/** Existing custom types are reused only when their schema is identical. */
export function prepareClipboardTypes(payload: SpaceClipboardPayload, existing: readonly SpaceKnowledgeObjectType[]) {
  const identities = new Map<string, string>();
  const definitions = payload.objectTypes.map((type) => {
    const target = existing.find((candidate) => candidate.id === type.id || candidate.name.toLowerCase() === type.name.toLowerCase());
    if (target && (target.system || target.name !== type.name || !isEqual(target.fields, type.fields))) {
      throw new Error(`The target Space already has a different custom type named '${type.name}'`);
    }
    const id = target?.id ?? crypto.randomUUID();
    identities.set(type.id, id);
    return { ...type, id };
  });
  return { identities, definitions };
}
''')
write("src/features/spaces/clipboard/fragment-preparation.ts", r'''
import { duplicateSpaceAsset, removeSpaceAssets } from "../assets/asset-operations";
import type { EditorFragment } from "../canvas/contract/editor-fragment";
import { prepareSpaceKnowledgePayload } from "../knowledge/model/node-knowledge";
import type { SpaceKnowledgeObjectType } from "../knowledge/model/object-types";
import type { SpaceNodePayload } from "../model/nodes/node-payload";
import { canonicalRelationEndpoints } from "../model/relations/relation";
import { cloneNodeData } from "./node-cloning";
import { prepareClipboardTypes } from "./fragment-types";
import { parseSpaceClipboardPayload } from "./payload-schema";
import type { SpaceClipboardPayload } from "./payload-types";

export async function prepareClipboardFragment(source: SpaceClipboardPayload, targetSpaceId: string, targetTypes: readonly SpaceKnowledgeObjectType[]) {
  const payload = parseSpaceClipboardPayload(source);
  const identities = new Map(payload.structure.items.map((item) => [item.id, crypto.randomUUID()]));
  const relationshipIds = new Map(payload.relations.map((item) => [item.id, crypto.randomUUID()]));
  const types = prepareClipboardTypes(payload, targetTypes);
  const assets: string[] = [];
  try {
    const context = { identities, assets, sourceSpaceId: payload.sourceSpaceId, targetSpaceId };
    const contents = new Map<string, SpaceNodePayload>();
    for (const node of payload.nodes) {
      // oxlint-disable-next-line no-await-in-loop -- Each staged file has one tracked cleanup owner.
      const data = await cloneNodeData(node.data, context);
      remapObjectReferences(data, payload, identities, types.identities, targetSpaceId);
      contents.set(identities.get(node.id)!, prepareSpaceKnowledgePayload(data));
    }
    for (const [id, data] of contents) {
      if (data.kind !== "subtitle") continue;
      const video = contents.get(data.sourceVideoNodeId);
      if (video?.kind === "video" && video.sourceRevision) contents.set(id, prepareSpaceKnowledgePayload({ ...data, sourceVideoRevision: video.sourceRevision }));
    }
    const layout = { version: 1 as const, items: payload.layout.items.map((record) => ({ ...structuredClone(record), itemId: identities.get(record.itemId)! })) };
    for (const record of layout.items) {
      if (record.kind === "group" || !record.appearance?.background) continue;
      // oxlint-disable-next-line no-await-in-loop -- Track each background before another staged copy can fail.
      const path = await duplicateSpaceAsset(record.appearance.background);
      assets.push(path); record.appearance.background = path;
    }
    const fragment: EditorFragment = {
      structure: { version: 1, items: payload.structure.items.map((item) => ({ ...item,
        id: identities.get(item.id)!, parentId: item.parentId === null ? null : identities.get(item.parentId)!,
        ...(item.kind === "content" ? { nodeId: identities.get(item.nodeId)! } : {}),
      })) }, layout,
      nodes: [...contents].map(([id, data]) => ({ id, data, kind: data.kind, sourceChunks: [] })),
      objectTypes: types.definitions,
      relations: payload.relations.map((relation) => {
        const [sourceId, targetId] = canonicalRelationEndpoints(identities.get(relation.sourceId)!, identities.get(relation.targetId)!, relation.kind);
        return { id: relationshipIds.get(relation.id)!, sourceId, targetId, kind: relation.kind, rationale: relation.rationale, status: relation.status,
          ...(relation.confidence === undefined ? {} : { confidence: relation.confidence }) };
      }),
      relationStyles: payload.relations.flatMap((relation) => relation.color ? [{ relationId: relationshipIds.get(relation.id)!, color: relation.color }] : []),
    };
    return { fragment, assets };
  } catch (cause) { await discardClipboardAssets(assets, cause); throw cause; }
}
function remapObjectReferences(data: SpaceNodePayload, source: SpaceClipboardPayload, identities: ReadonlyMap<string, string>, types: ReadonlyMap<string, string>, targetSpaceId: string) {
  if (data.kind !== "object") return;
  const definition = source.objectTypes.find((type) => type.id === data.objectTypeId);
  data.objectTypeId = types.get(data.objectTypeId) ?? data.objectTypeId;
  for (const field of definition?.fields ?? []) {
    if (field.kind !== "node-ref") continue;
    const value = data.properties[field.key];
    if (typeof value !== "string" || !value) continue;
    if (identities.has(value)) data.properties[field.key] = identities.get(value)!;
    else if (source.sourceSpaceId !== targetSpaceId) throw new Error(`Copy the object referenced by '${field.label}' when pasting into another Space`);
  }
}
export async function discardClipboardAssets(paths: readonly string[], original?: unknown) {
  try { await removeSpaceAssets([...paths]); }
  catch (cleanup) { throw new AggregateError(original === undefined ? [cleanup] : [original, cleanup], "The edit was rejected and some staged files could not be cleaned up"); }
}
''')
write("src/features/spaces/clipboard/selection-transfer.ts", r'''
import { useCanvasEditorSession } from "../canvas/editor-session";
import type { CanvasSelectionTransfer } from "../canvas/contract/canvas-editor";
import { EditorOutcomeUnknownError } from "../canvas/persistence/editor-command-repository";
import { unionStructureBounds } from "../canvas/document/structure-transform";
import { listObjectTypes } from "../knowledge/persistence/knowledge-repository";
import { scheduleSpaceAssetProcessing } from "../assets/processing-scheduler";
import { isSpaceManagedAssetPayload } from "../model/nodes/capabilities";
import { indexStructureSnapshot } from "../organization/structure/structure-context";
import { projectStructure } from "../organization/structure/structure-projection";
import { structureOperationRoots } from "../organization/structure/structure-selection";
import type { useSpaceStore } from "../session/space-store";
import { createSpaceClipboardPayload } from "./payload";
import type { SpaceClipboardPayload } from "./payload-types";
import { prepareClipboardFragment, discardClipboardAssets } from "./fragment-preparation";
import { writeSpaceClipboardDirect, readSpaceClipboardHtml } from "./transfer";

type Store = ReturnType<typeof useSpaceStore>;
export async function selectionClipboardPayload(store: Store, transfer: CanvasSelectionTransfer) {
  const spaceId = store.activeSpaceId;
  if (!spaceId) throw new Error("An active Space is required to copy content");
  const nodes = [...store.nodes], relations = [...store.relations];
  const objectTypes = await listObjectTypes(spaceId);
  return createSpaceClipboardPayload({ spaceId, transfer, nodes, relations, objectTypes });
}
export async function copySpaceSelectionToClipboard(store: Store, transfer: CanvasSelectionTransfer) {
  if (transfer.itemIds.length) await writeSpaceClipboardDirect(await selectionClipboardPayload(store, transfer));
}
export async function duplicateSpaceSelection(store: Store, transfer: CanvasSelectionTransfer) {
  if (!transfer.itemIds.length) return;
  const payload = await selectionClipboardPayload(store, transfer);
  const context = indexStructureSnapshot(transfer.document);
  const roots = structureOperationRoots(context.tree, transfer.itemIds);
  const parents = new Set(roots.map((item) => item.parentId));
  const parentId = parents.size === 1 ? [...parents][0]! : null;
  await insertClipboardSelection(store, payload, parentId, { x: 24, y: 24 });
}
export async function pasteSpaceClipboardAt(store: Store, point: { x: number; y: number }, html: string | null) {
  const payload = html ? readSpaceClipboardHtml(html) : undefined;
  if (!payload) return false;
  const context = indexStructureSnapshot(payload); const projection = projectStructure(context);
  const bounds = unionStructureBounds((context.tree.children.get(null) ?? []).map((item) => projection.bounds.get(item.id)!))!;
  const parentId = useCanvasEditorSession().selection.value.scopeGroupId ?? null;
  if (parentId && payload.structure.items.some((item) => item.kind === "section")) throw new Error("Exit the Group before pasting a Section at the Space root");
  await insertClipboardSelection(store, payload, parentId, { x: point.x - bounds.x, y: point.y - bounds.y });
  return true;
}
async function insertClipboardSelection(store: Store, source: SpaceClipboardPayload, parentId: string | null, delta: { x: number; y: number }) {
  const session = useCanvasEditorSession(); const editor = session.activeEditor.value;
  const spaceId = store.activeSpaceId;
  if (!spaceId || !editor) throw new Error("An active editor is required for pasting");
  await store.ensureActiveSpacePersisted();
  const prepared = await prepareClipboardFragment(source, spaceId, await listObjectTypes(spaceId));
  try {
    if (store.activeSpaceId !== spaceId || session.activeEditor.value !== editor) throw new Error("The paste destination changed while resources were being prepared");
    await editor.insertFragment(prepared.fragment, { parentId, delta });
  } catch (cause) {
    if (!(cause instanceof EditorOutcomeUnknownError)) await discardClipboardAssets(prepared.assets, cause);
    throw cause;
  }
  // Only confirmed committed content may start enrichment. Never clean committed files on readback errors.
  for (const node of prepared.fragment.nodes) {
    if (isSpaceManagedAssetPayload(node.data)) scheduleSpaceAssetProcessing({ applyNodeDataForSpace: store.applyNodeDataForSpace }, spaceId, node.id, node.data);
  }
}
''')
# Existing product entrypoints now pass a complete captured native selection rather than positions only.
path = "src/features/spaces/session/actions/canvas-selection-actions.ts"
source = read(path)
source = source.replace('from "../../canvas/selection-actions"', 'from "../../clipboard/selection-transfer"')
source = source.replace('CanvasNodePlacement', 'CanvasSelectionTransfer')
source = source.replace('placements: readonly CanvasSelectionTransfer[]', 'transfer: CanvasSelectionTransfer')
source = source.replace('nodes: readonly CanvasSelectionTransfer[]', 'transfer: CanvasSelectionTransfer')
source = source.replace('copySpaceSelectionToClipboard(positionedSelection(store.nodes, placements))', 'copySpaceSelectionToClipboard(store, transfer)')
source = source.replace('duplicateSpaceSelection(store, positionedSelection(store.nodes, placements))', 'duplicateSpaceSelection(store, transfer)')
# Replace the public wrappers structurally, not the native history implementation pending its removal.
source = re.sub(r'copyNodes:\s*\([^)]*\)\s*=>\s*copySpaceSelectionToClipboard\([^;]*?\),\n', 'copyNodes: (transfer: CanvasSelectionTransfer) => copySpaceSelectionToClipboard(store, transfer),\n', source)
source = re.sub(r'duplicateNodes:\s*\([^)]*\)\s*=>\s*duplicateSpaceSelection\([^;]*?\),\n', 'duplicateNodes: (transfer: CanvasSelectionTransfer) => duplicateSpaceSelection(store, transfer),\n', source)
# Native selection deletion already owns cut; obsolete host cut wrappers are removed in the boundary cleanup stage.
if 'function positionedSelection(' in source:
    source = source[:source.index('function positionedSelection(')]
source = source.replace('import type { PositionedSpaceNode, SpaceCanvasNode } from "../../model/nodes/canvas-node";', 'import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";')
write(path, source)
path = "src/features/spaces/canvas/selection-actions.ts"
source = read(path)
start = source.index("export async function duplicateSpaceSelection(")
end = source.index("type OpenHandler", start)
source = source[:start] + source[end:]
start = source.find("\nfunction scheduleManagedCopies(")
if start >= 0: source = source[:start]
for line in (
    'import { removeSpaceAssets } from "../assets/asset-operations";\n',
    'import { assetPathsForNodes } from "../assets/node-asset-paths";\n',
    'import { scheduleSpaceAssetProcessing } from "../assets/processing-scheduler";\n',
    'import { cloneClipboardNodes, duplicateSpaceNodes } from "../clipboard/node-cloning";\n',
    'import { readSpaceClipboardHtml, writeSpaceClipboardDirect } from "../clipboard/transfer";\n',
): source = source.replace(line, '')
source = source.replace('PositionedSpaceNode, ', '')
write(path, source)
replace("src/features/spaces/presentation/canvas/use-space-canvas-imports.ts", 'from "../../canvas/selection-actions"', 'from "../../clipboard/selection-transfer"')
# Ensure copied custom types match a reused target definition atomically, not only at preparation time.
write("src-tauri/src/spaces/editing/fragment_types.rs", r'''
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
''')
for path, content in sources.items():
    destination = root / path
    destination.parent.mkdir(parents=True, exist_ok=True)
    destination.write_text(content)
marker.parent.mkdir(parents=True, exist_ok=True)
marker.write_text("Reviewed structural clipboard integration applied. Remove development artifacts before delivery.\n")
print(f"Applied native clipboard integration to {len(sources)} source files")

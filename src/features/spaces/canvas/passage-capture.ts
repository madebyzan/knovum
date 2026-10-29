import { appendPlainText } from "../document-editing/tiptap-content";
import type { ProvenanceSource } from "../knowledge/model/provenance";
import { attachProvenance } from "../knowledge/provenance-actions";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceNodePayload } from "../model/nodes/node-payload";
import { useSpaceStore } from "../session/space-store";
import type { CanvasPoint } from "./contracts/canvas-editor-port";
import { canvasLayoutBounds } from "./document/canvas-layout-bounds";
import { currentCanvasEditor } from "./editor-session";
import type { CanvasNodeLayoutV1 } from "./schema/document";

/** Room between a passage and the items it came from, in canvas units. */
const GAP = 48;
/** How far each further passage beside the same items steps down, so none hides another. */
const CASCADE = 32;

type SpaceStore = ReturnType<typeof useSpaceStore>;

/** Where a passage goes: right of the items it came from, top-aligned, below earlier passages there. */
export function besideNodes(layouts: readonly CanvasNodeLayoutV1[], nodeIds: readonly string[]) {
  const ids = new Set(nodeIds);
  const bounds = canvasLayoutBounds(layouts.filter((layout) => ids.has(layout.nodeId)));
  if (!bounds) return undefined;
  const x = bounds.x + bounds.width + GAP;
  const taken = new Set(
    layouts.filter(({ frame }) => Math.abs(frame.x - x) < 1).map(({ frame }) => Math.round(frame.y)),
  );
  let y = bounds.y;
  while (taken.has(Math.round(y))) y += CASCADE;
  return { x, y };
}

/** Whether a passage can be added to the item: the kinds a person goes on editing as text. */
export function acceptsPassage(data: SpaceNodePayload) {
  return data.kind === "note" || data.kind === "markdown";
}

/** The item's content with `text` added at its end. */
export function withPassage(data: SpaceNodePayload, text: string): SpaceNodePayload | undefined {
  if (data.kind === "note") return appendPlainText(data, text);
  if (data.kind !== "markdown") return undefined;
  const before = data.text.trimEnd();
  return { ...data, text: before ? `${before}\n\n${text}` : text };
}

/**
 * Puts `payload` on the canvas beside its sources and links it to them, as one undoable step. What
 * it was taken from is kept with it, so a later change to a source shows on the item.
 */
export async function addPassageNode(
  payload: SpaceNodePayload,
  sources: readonly ProvenanceSource[],
  position?: CanvasPoint,
) {
  const store = useSpaceStore();
  const spaceId = activeSpace(store);
  const taken = liveSources(store, sources);
  const sourceIds = taken.map(({ nodeId }) => nodeId);
  const layouts = currentCanvasEditor()?.getDocument().nodes ?? [];
  let added: SpaceCanvasNode | undefined;
  await store.recordHistoryMutation(async () => {
    [added] = await store.addContent(payload, position ?? besideNodes(layouts, sourceIds));
    if (added) await linkSources(store, spaceId, added.id, sourceIds);
  });
  if (added) await keepProvenance(spaceId, added.id, taken);
  return added;
}

/** Adds `text` to the end of an existing note and links the note to its sources, as one step. */
export async function appendPassage(nodeId: string, text: string, sources: readonly ProvenanceSource[]) {
  const store = useSpaceStore();
  const spaceId = activeSpace(store);
  const node = store.nodes.find((candidate) => candidate.id === nodeId);
  const after = node && withPassage(node.data, text);
  if (!node || !after) throw new Error(`Canvas item ${nodeId} cannot take a passage`);
  const taken = liveSources(store, sources).filter((source) => source.nodeId !== nodeId);
  await store.recordHistoryMutation(async () => {
    await store.updateNodeContents([{ id: nodeId, before: node.data, after }]);
    await linkSources(
      store,
      spaceId,
      nodeId,
      taken.map((source) => source.nodeId),
    );
  });
  await keepProvenance(spaceId, nodeId, taken);
  return node;
}

// A cited item may have been deleted since the answer was written; a link to it would be refused.
// One passage per source is kept: provenance holds a single current record for each source.
function liveSources(store: SpaceStore, sources: readonly ProvenanceSource[]) {
  const live = new Set(store.nodes.filter((node) => !node.hidden).map((node) => node.id));
  const first = new Map<string, ProvenanceSource>();
  for (const source of sources)
    if (live.has(source.nodeId) && !first.has(source.nodeId)) first.set(source.nodeId, source);
  return [...first.values()];
}

function activeSpace(store: SpaceStore) {
  if (!store.activeSpaceId) throw new Error("A Space must be active to keep a passage");
  return store.activeSpaceId;
}

// The Space is fixed when the passage is taken, so links never land in one opened meanwhile.
async function linkSources(store: SpaceStore, spaceId: string, nodeId: string, sources: readonly string[]) {
  await Promise.all(
    sources.map((source) => store.createRelation(nodeId, source, "derives-from", { spaceId })),
  );
}

// Provenance is knowledge about the item, not an edit of the canvas: undo leaves it, and a restored
// item still knows its sources.
async function keepProvenance(spaceId: string, targetId: string, sources: readonly ProvenanceSource[]) {
  await Promise.all(
    sources.map((source) => attachProvenance({ spaceId, targetKind: "node", targetId }, source)),
  );
}

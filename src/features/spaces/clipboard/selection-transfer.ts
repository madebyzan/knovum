import { scheduleSpaceAssetProcessing } from "../assets/processing-scheduler";
import type { CanvasSelectionTransfer } from "../canvas/contract/canvas-editor";
import { unionStructureBounds } from "../canvas/document/structure-transform";
import { useCanvasEditorSession } from "../canvas/editor-session";
import { EditorOutcomeUnknownError } from "../canvas/persistence/editor-command-repository";
import { listObjectTypes } from "../knowledge/persistence/knowledge-repository";
import { isSpaceManagedAssetPayload } from "../model/nodes/capabilities";
import { indexStructureSnapshot } from "../organization/structure/structure-context";
import { projectStructure } from "../organization/structure/structure-projection";
import { structureOperationRoots } from "../organization/structure/structure-selection";
import type { useSpaceStore } from "../session/space-store";
import { prepareClipboardFragment, discardClipboardAssets } from "./fragment-preparation";
import { createSpaceClipboardPayload } from "./payload";
import type { SpaceClipboardPayload } from "./payload-types";
import { writeSpaceClipboardDirect, readSpaceClipboardHtml } from "./transfer";

type Store = ReturnType<typeof useSpaceStore>;
export async function selectionClipboardPayload(store: Store, transfer: CanvasSelectionTransfer) {
  const spaceId = store.activeSpaceId;
  if (!spaceId) throw new Error("An active Space is required to copy content");
  const nodes = [...store.nodes],
    relations = [...store.relations];
  const objectTypes = await listObjectTypes(spaceId);
  return createSpaceClipboardPayload({ spaceId, transfer, nodes, relations, objectTypes });
}
export async function copySpaceSelectionToClipboard(store: Store, transfer: CanvasSelectionTransfer) {
  if (transfer.itemIds.length)
    await writeSpaceClipboardDirect(await selectionClipboardPayload(store, transfer));
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
export async function pasteSpaceClipboardAt(
  store: Store,
  point: { x: number; y: number },
  html: string | null,
) {
  const payload = html ? readSpaceClipboardHtml(html) : undefined;
  if (!payload) return false;
  const context = indexStructureSnapshot(payload);
  const projection = projectStructure(context);
  const bounds = unionStructureBounds(
    (context.tree.children.get(null) ?? []).map((item) => projection.bounds.get(item.id)!),
  )!;
  const parentId = useCanvasEditorSession().selection.value.scopeGroupId ?? null;
  if (parentId && payload.structure.items.some((item) => item.kind === "section"))
    throw new Error("Exit the Group before pasting a Section at the Space root");
  await insertClipboardSelection(store, payload, parentId, { x: point.x - bounds.x, y: point.y - bounds.y });
  return true;
}
async function insertClipboardSelection(
  store: Store,
  source: SpaceClipboardPayload,
  parentId: string | null,
  delta: { x: number; y: number },
) {
  const session = useCanvasEditorSession();
  const editor = session.activeEditor.value;
  const spaceId = store.activeSpaceId;
  if (!spaceId || !editor) throw new Error("An active editor is required for pasting");
  await store.ensureActiveSpacePersisted();
  const prepared = await prepareClipboardFragment(source, spaceId, await listObjectTypes(spaceId));
  try {
    if (store.activeSpaceId !== spaceId || session.activeEditor.value !== editor)
      throw new Error("The paste destination changed while resources were being prepared");
    await editor.insertFragment(prepared.fragment, { parentId, delta });
  } catch (cause) {
    if (!(cause instanceof EditorOutcomeUnknownError)) await discardClipboardAssets(prepared.assets, cause);
    throw cause;
  }
  // Only confirmed committed content may start enrichment. Never clean committed files on readback errors.
  for (const node of prepared.fragment.nodes) {
    if (isSpaceManagedAssetPayload(node.data))
      scheduleSpaceAssetProcessing(
        { applyNodeDataForSpace: store.applyNodeDataForSpace },
        spaceId,
        node.id,
        node.data,
      );
  }
}

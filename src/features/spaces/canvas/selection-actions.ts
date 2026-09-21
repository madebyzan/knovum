import { openUrl } from "@tauri-apps/plugin-opener";

import { openSystemPreview } from "../../../foundation/system-preview/system-preview";
import type { SpaceClipboardPayload } from "../clipboard/payload-types";
import { importStructureClipboard } from "../clipboard/structure-clipboard-repository";
import { readSpaceClipboardHtml, writeSpaceClipboardDirect } from "../clipboard/transfer";
import type { PositionedSpaceNode, SpaceCanvasNode } from "../model/nodes/canvas-node";
import { isSpaceManagedAssetPayload } from "../model/nodes/capabilities";
import type { SpaceNodeKind } from "../model/nodes/node-payload";
import { createStructureIndex } from "../model/structure/structure-index";
import { groupScopePathFor } from "../model/structure/structure-queries";
import type { useSpaceStore } from "../session/space-store";
import { useCanvasEditorSession } from "./editor-session";

export async function duplicateSpaceSelection(
  store: ReturnType<typeof useSpaceStore>,
  selectedNodes: readonly PositionedSpaceNode[],
) {
  const structureNodeIdByContentId = new Map(
    store.editSnapshot.structure.flatMap((structureNode) =>
      structureNode.kind === "content" ? [[structureNode.nodeId, structureNode.id] as const] : [],
    ),
  );
  const rootIds = selectedNodes.map(({ node }) => {
    const structureNodeId = structureNodeIdByContentId.get(node.id);
    if (!structureNodeId)
      throw new Error(`Duplicate content is unavailable in the Space structure: ${node.id}`);
    return structureNodeId;
  });
  if (!rootIds.length) return;
  await store.executeSpaceEdit({
    kind: "structure.clone",
    rootIds,
    targetParentId: null,
    offset: { x: 24, y: 24 },
  });
}

export async function copySpaceSelectionToClipboard(payload: SpaceClipboardPayload) {
  await writeSpaceClipboardDirect(payload);
}

export async function pasteSpaceClipboardAt(
  store: ReturnType<typeof useSpaceStore>,
  point: { x: number; y: number },
  html: string | null,
) {
  if (html) {
    const payload = readSpaceClipboardHtml(html);
    if (!payload) return false;
    await store.ensureActiveSpacePersisted();
    const spaceId = store.activeSpaceId;
    if (!spaceId) throw new Error("A Space must be active to paste structured content");
    const result = await importStructureClipboard({
      spaceId,
      mutationId: crypto.randomUUID(),
      expectedEditRevision: store.editSnapshot.editRevision,
      targetParentId: store.groupScopePath.at(-1) ?? null,
      anchor: point,
      payload,
    });
    if (store.activeSpaceId !== spaceId) throw new Error("Structured clipboard paste target changed");
    store.nodes = [...store.nodes, ...result.nodes];
    store.relations = [...store.relations, ...result.relations];
    store.editSnapshot = result.snapshot;
    store.recordCommittedSpaceEdit();
    useCanvasEditorSession().selectStructureNodes(result.selectionSuggestion);
    return true;
  }
  return false;
}

type OpenHandler = (store: ReturnType<typeof useSpaceStore>, node: SpaceCanvasNode) => void | Promise<void>;

function openEditorNode(store: ReturnType<typeof useSpaceStore>, node: SpaceCanvasNode) {
  store.focusedEditorId = node.id;
}

const OPEN_HANDLERS: Partial<Record<SpaceNodeKind, OpenHandler>> = {
  note: openEditorNode,
  markdown: openEditorNode,
  website: (_, node) => {
    if (node.data.kind === "website" && node.data.url) return openUrl(node.data.url);
  },
  portal: (store, node) => {
    if (node.data.kind !== "portal") return;
    return openPortalNode(store, node.data);
  },
};

export async function openSpaceNode(store: ReturnType<typeof useSpaceStore>, node?: SpaceCanvasNode) {
  if (!node) return;
  const handler = OPEN_HANDLERS[node.data.kind];
  if (handler) {
    await handler(store, node);
    return;
  }
  if (isSpaceManagedAssetPayload(node.data)) await openSystemPreview(node.data.assetPath);
}

export function isSpaceNodeOpenable(node?: SpaceCanvasNode) {
  return Boolean(
    node &&
    (["note", "markdown", "website", "portal"].includes(node.data.kind) ||
      isSpaceManagedAssetPayload(node.data)),
  );
}

export function selectAndRevealSpaceNode(id: string, mode: "focus" | "ensure-visible" = "focus") {
  useCanvasEditorSession().reveal([id], mode);
}

export async function openPortalNode(
  store: ReturnType<typeof useSpaceStore>,
  target: Extract<SpaceCanvasNode["data"], { kind: "portal" }>,
) {
  const {
    targetSpaceId: spaceId,
    targetSceneId: sceneId,
    targetNodeId: nodeId,
    targetStructureNodeId: structureNodeId,
  } = target;
  await store.openSpace(spaceId);
  if (sceneId) await store.openSceneView(sceneId);
  const editor = useCanvasEditorSession();
  if (nodeId) editor.reveal([nodeId], "focus");
  if (!structureNodeId) return;
  const index = createStructureIndex(store.editSnapshot.structure);
  const structureNode = index.structureById.get(structureNodeId);
  if (!structureNode || structureNode.kind === "content")
    throw new Error("Portal structure target is unavailable");
  store.groupScopePath = groupScopePathFor(structureNodeId, index);
  editor.selectStructureNodes([structureNodeId]);
  editor.reveal([structureNodeId], "focus");
}

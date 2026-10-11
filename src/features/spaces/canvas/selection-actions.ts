import { openUrl } from "@tauri-apps/plugin-opener";

import { invokeCommand } from "../../../foundation/ipc/invoke-command";
import { previewSpaceAsset } from "../assets/asset-preview";
import type { SpaceClipboardPayload } from "../clipboard/clipboard-envelope";
import { importStructureClipboard } from "../clipboard/structure-clipboard-operations";
import { readSpaceClipboardHtml, writeSpaceClipboardDirect } from "../clipboard/transfer";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import { isSpaceManagedAssetPayload } from "../model/nodes/capabilities";
import type { SpaceNodeKind } from "../model/nodes/node-payload";
import { createStructureIndex } from "../model/structure/structure-index";
import { groupScopePathFor } from "../model/structure/structure-queries";
import { useSpaceStore } from "../session/space-store";
import { getCanvasEditorSession } from "./editor-session";

export async function copySpaceSelectionToClipboard(payload: SpaceClipboardPayload) {
  await writeSpaceClipboardDirect(payload);
}

export async function pasteSpaceClipboardAt(point: { x: number; y: number }, html: string | null) {
  const store = useSpaceStore();

  if (html) {
    const payload = readSpaceClipboardHtml(html);
    if (!payload) return false;
    await store.ensureActiveSpacePersisted();
    const spaceId = store.activeSpaceId;
    if (!spaceId) throw new Error("A Space must be active to paste structured content");
    const pasteReceipt = await importStructureClipboard({
      spaceId,
      mutationId: crypto.randomUUID(),
      expectedEditRevision: store.editSnapshot.editRevision,
      targetParentId: store.groupScopePath.at(-1) ?? null,
      anchor: point,
      payload,
    });
    if (store.activeSpaceId !== spaceId) throw new Error("Structured clipboard paste target changed");
    store.nodes = [...store.nodes, ...pasteReceipt.nodes];
    store.relations = [...store.relations, ...pasteReceipt.relations];
    store.editSnapshot = pasteReceipt.snapshot;
    store.recordCommittedSpaceEdit();
    getCanvasEditorSession().selectStructureNodes(pasteReceipt.selectionSuggestion);
    return true;
  }
  return false;
}

type OpenHandler = (node: SpaceCanvasNode) => void | Promise<void>;

function openEditorNode(node: SpaceCanvasNode) {
  const store = useSpaceStore();
  store.focusedEditorId = node.id;
}

const OPEN_HANDLERS: Partial<Record<SpaceNodeKind, OpenHandler>> = {
  note: openEditorNode,
  markdown: openEditorNode,
  website: (node) => {
    if (node.data.kind === "website" && node.data.url) return openUrl(node.data.url);
  },
  portal: (node) => {
    if (node.data.kind !== "portal") return;
    return openPortalNode(node.data);
  },
};

export async function openSpaceNode(node?: SpaceCanvasNode) {
  const store = useSpaceStore();

  if (!node || !isSpaceNodeOpenable(node)) return;
  const handler = OPEN_HANDLERS[node.data.kind];
  if (handler) {
    await handler(node);
    return;
  }
  if (node.data.kind === "image") {
    const spaceId = store.activeSpaceId;
    if (!spaceId) throw new Error("Image Space is unavailable");
    await invokeCommand("plugin:spaces|preview_image", { spaceId, nodeId: node.id });
    return;
  }
  if (isSpaceManagedAssetPayload(node.data)) await previewSpaceAsset(node.data.assetPath);
}

export function isSpaceNodeOpenable(node?: SpaceCanvasNode) {
  if (!node) return false;
  if (node.data.kind === "video" && node.data.processingState && !node.data.sourceRevision) return false;
  return OPEN_HANDLERS[node.data.kind] !== undefined || isSpaceManagedAssetPayload(node.data);
}

export function selectAndRevealSpaceNode(nodeId: string, mode: "focus" | "ensure-visible" = "focus") {
  getCanvasEditorSession().reveal([nodeId], mode);
}

async function openPortalNode(target: Extract<SpaceCanvasNode["data"], { kind: "portal" }>) {
  const store = useSpaceStore();
  const {
    targetSpaceId: spaceId,
    targetSceneId: sceneId,
    targetNodeId: nodeId,
    targetStructureNodeId: structureNodeId,
  } = target;
  await store.openSpace(spaceId);
  if (sceneId) await store.openSceneView(sceneId);
  const editor = getCanvasEditorSession();
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

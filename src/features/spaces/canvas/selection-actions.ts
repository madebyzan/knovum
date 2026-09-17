import { openUrl } from "@tauri-apps/plugin-opener";

import { openSystemPreview } from "../../../foundation/system-preview/system-preview";
import { removeSpaceAssets } from "../assets/asset-operations";
import { assetPathsForNodes } from "../assets/node-asset-paths";
import { scheduleSpaceAssetProcessing } from "../assets/processing-scheduler";
import { cloneClipboardNodes, duplicateSpaceNodes } from "../clipboard/node-cloning";
import { readSpaceClipboardHtml, writeSpaceClipboardDirect } from "../clipboard/transfer";
import type { PositionedSpaceNode, SpaceCanvasNode } from "../model/nodes/canvas-node";
import { isSpaceManagedAssetPayload, isSpaceSectionPayload } from "../model/nodes/capabilities";
import type { SpaceNodeKind } from "../model/nodes/node-payload";
import type { useSpaceStore } from "../session/space-store";
import { useCanvasEditorSession } from "./editor-session";

export async function duplicateSpaceSelection(
  store: ReturnType<typeof useSpaceStore>,
  selectedNodes: readonly PositionedSpaceNode[],
) {
  const copies = await duplicateSpaceNodes(selectedNodes);
  if (!copies.length) return;
  try {
    await store.addItems(copies);
  } catch (error) {
    await removeSpaceAssets(assetPathsForNodes(copies));
    throw error;
  }
  scheduleManagedCopies(store, copies);
  useCanvasEditorSession().selectNodes(copies.map((node) => node.id));
}

export async function copySpaceSelectionToClipboard(selectedNodes: readonly PositionedSpaceNode[]) {
  await writeSpaceSelectionToClipboard(selectedNodes);
}

export async function pasteSpaceClipboardAt(
  store: ReturnType<typeof useSpaceStore>,
  point: { x: number; y: number },
  html: string | null,
) {
  if (html) {
    const payload = readSpaceClipboardHtml(html);
    if (!payload) return false;
    const copies = await cloneClipboardNodes(payload, point);
    try {
      await store.addItems(copies);
    } catch (error) {
      await removeSpaceAssets(assetPathsForNodes(copies));
      throw error;
    }
    scheduleManagedCopies(store, copies);
    return true;
  }
  return false;
}

async function writeSpaceSelectionToClipboard(nodes: readonly PositionedSpaceNode[]) {
  await writeSpaceClipboardDirect(nodes);
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
    return openPortalNode(store, node.data.targetSpaceId, node.data.targetSceneId, node.data.targetNodeId);
  },
};

export async function openSpaceNode(store: ReturnType<typeof useSpaceStore>, node?: SpaceCanvasNode) {
  if (!node || isSpaceSectionPayload(node.data)) return;
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
  spaceId: string,
  sceneId?: string,
  nodeId?: string,
) {
  await store.openSpace(spaceId);
  if (sceneId) await store.openSceneView(sceneId);
  if (nodeId) useCanvasEditorSession().reveal([nodeId], "focus");
}

function scheduleManagedCopies(store: ReturnType<typeof useSpaceStore>, nodes: readonly SpaceCanvasNode[]) {
  const spaceId = store.activeSpaceId;
  if (!spaceId) return;
  const context = { applyNodeDataForSpace: store.applyNodeDataForSpace };
  for (const node of nodes) {
    if (!isSpaceManagedAssetPayload(node.data)) continue;
    scheduleSpaceAssetProcessing(context, spaceId, node.id, node.data);
  }
}

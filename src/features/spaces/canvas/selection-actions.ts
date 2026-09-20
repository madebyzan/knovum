import { openUrl } from "@tauri-apps/plugin-opener";

import { openSystemPreview } from "../../../foundation/system-preview/system-preview";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import { isSpaceManagedAssetPayload } from "../model/nodes/capabilities";
import type { SpaceNodeKind } from "../model/nodes/node-payload";
import type { useSpaceStore } from "../session/space-store";
import { useCanvasEditorSession } from "./editor-session";

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
  spaceId: string,
  sceneId?: string,
  nodeId?: string,
) {
  await store.openSpace(spaceId);
  if (sceneId) await store.openSceneView(sceneId);
  if (nodeId) useCanvasEditorSession().reveal([nodeId], "focus");
}

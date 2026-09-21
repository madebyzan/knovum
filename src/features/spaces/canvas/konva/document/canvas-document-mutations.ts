import type {
  CanvasDocumentV1,
  CanvasNodeFrame,
  CanvasNodeFrameUpdate,
} from "../../contract/canvas-document";
import type { CanvasGestureUpdate } from "../interaction/canvas-gesture-update";

export function applyCanvasFrames(
  document: CanvasDocumentV1,
  updates: readonly CanvasNodeFrameUpdate[],
): CanvasDocumentV1 {
  const frames = new Map(updates.map((update) => [update.nodeId, update]));
  return {
    ...document,
    nodes: document.nodes.map((node) => {
      const update = frames.get(node.nodeId);
      if (!update) return node;
      const sizeAuthority = update.sizeAuthority ?? node.sizeAuthority;
      if (sameCanvasFrame(node.frame, update.frame) && sizeAuthority === node.sizeAuthority) return node;
      return { ...node, frame: { ...update.frame }, sizeAuthority };
    }),
  };
}

export function applyCanvasGestureUpdates(
  document: CanvasDocumentV1,
  updates: readonly CanvasGestureUpdate[],
) {
  const layouts = new Map(updates.map((update) => [update.nodeId, update]));
  return {
    ...document,
    nodes: document.nodes.map((node) => {
      const update = layouts.get(node.nodeId);
      if (!update) return node;
      return {
        ...node,
        frame: { ...update.frame },
        sizeAuthority: update.sizeAuthority ?? node.sizeAuthority,
        appearance: "appearance" in update ? (update.appearance ?? undefined) : node.appearance,
      };
    }),
  };
}

export function captureCanvasGestureLayouts(
  document: CanvasDocumentV1,
  updates: readonly CanvasGestureUpdate[],
) {
  const ids = new Set(updates.map(({ nodeId }) => nodeId));
  return document.nodes.flatMap<CanvasGestureUpdate>(({ nodeId, frame, sizeAuthority, appearance }) =>
    ids.has(nodeId) ? [{ nodeId, frame: { ...frame }, sizeAuthority, appearance: appearance ?? null }] : [],
  );
}

export function sameCanvasFrame(left: CanvasNodeFrame, right: CanvasNodeFrame) {
  return (
    left.x === right.x &&
    left.y === right.y &&
    left.width === right.width &&
    left.height === right.height &&
    left.rotation === right.rotation
  );
}

export function sameCanvasViewport(left: CanvasDocumentV1["viewport"], right: CanvasDocumentV1["viewport"]) {
  return left.x === right.x && left.y === right.y && left.zoom === right.zoom;
}

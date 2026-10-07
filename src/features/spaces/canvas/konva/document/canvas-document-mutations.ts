import type { CanvasDocumentV1 } from "../../schema/document";
import type { CanvasGestureUpdate } from "../interaction/canvas-gesture-update";

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
        sizingMode: update.sizingMode ?? node.sizingMode,
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
  return document.nodes.flatMap<CanvasGestureUpdate>(({ nodeId, frame, sizingMode, appearance }) =>
    ids.has(nodeId) ? [{ nodeId, frame: { ...frame }, sizingMode, appearance: appearance ?? null }] : [],
  );
}

export function sameCanvasViewport(left: CanvasDocumentV1["viewport"], right: CanvasDocumentV1["viewport"]) {
  return left.x === right.x && left.y === right.y && left.zoom === right.zoom;
}

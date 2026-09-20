import type {
  CanvasNodeFrame,
  CanvasNodeFrameUpdate,
  SpaceEditorDocument,
} from "../../contract/canvas-document";
import { applyEditorFrames } from "../../document/editor-frame-edit";
import { projectEditorLayouts } from "../../document/editor-layout-projection";
import type { CanvasGestureUpdate } from "../interaction/canvas-gesture-update";
export function applyCanvasFrames(document: SpaceEditorDocument, updates: readonly CanvasNodeFrameUpdate[]) {
  return applyEditorFrames(document, updates);
}
export function applyCanvasGestureUpdates(
  document: SpaceEditorDocument,
  updates: readonly CanvasGestureUpdate[],
) {
  const result = applyEditorFrames(document, updates);
  const byId = new Map(updates.map((update) => [update.nodeId, update]));
  return {
    ...result,
    layout: {
      version: 1 as const,
      items: result.layout.items.map((layout) => {
        const update = byId.get(layout.itemId);
        if (!update || !("appearance" in update) || layout.kind === "group") return layout;
        const { appearance: _appearance, ...rest } = layout;
        return update.appearance ? { ...rest, appearance: update.appearance } : rest;
      }),
    },
  };
}
export function captureCanvasGestureLayouts(
  document: SpaceEditorDocument,
  updates: readonly CanvasGestureUpdate[],
) {
  const selected = new Set(updates.map((update) => update.nodeId));
  return projectEditorLayouts(document).filter((layout) => selected.has(layout.nodeId));
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
export function sameCanvasViewport(
  left: SpaceEditorDocument["viewport"],
  right: SpaceEditorDocument["viewport"],
) {
  return left.x === right.x && left.y === right.y && left.zoom === right.zoom;
}

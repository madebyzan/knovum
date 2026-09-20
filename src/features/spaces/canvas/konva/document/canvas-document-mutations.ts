import { sameCanvasNodeAppearance } from "../../contract/canvas-document";
import type {
  SpaceEditorDocument,
  CanvasNodeFrame,
  CanvasNodeFrameUpdate,
} from "../../contract/canvas-document";
import type { CanvasGestureUpdate } from "../interaction/canvas-gesture-update";

export function applyCanvasFrames(
  document: SpaceEditorDocument,
  updates: readonly CanvasNodeFrameUpdate[],
): SpaceEditorDocument {
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
  document: SpaceEditorDocument,
  updates: readonly CanvasGestureUpdate[],
) {
  const layouts = new Map(updates.map((update) => [update.nodeId, update]));
  return {
    ...document,
    nodes: document.nodes.map((node) => {
      const update = layouts.get(node.nodeId);
      if (!update) return node;
      const sizeAuthority = update.sizeAuthority ?? node.sizeAuthority;
      const appearance = "appearance" in update ? (update.appearance ?? undefined) : node.appearance;
      if (
        sameCanvasFrame(node.frame, update.frame) &&
        sizeAuthority === node.sizeAuthority &&
        sameCanvasNodeAppearance(node.appearance, appearance)
      )
        return node;
      const next = { ...node, frame: { ...update.frame }, sizeAuthority };
      if (appearance) next.appearance = { ...appearance };
      else delete next.appearance;
      return next;
    }),
  };
}

export function captureCanvasGestureLayouts(
  document: SpaceEditorDocument,
  updates: readonly CanvasGestureUpdate[],
) {
  const ids = new Set(updates.map(({ nodeId }) => nodeId));
  return document.nodes.flatMap<CanvasGestureUpdate>(({ nodeId, frame, sizeAuthority, appearance }) =>
    ids.has(nodeId) ? [{ nodeId, frame: { ...frame }, sizeAuthority, appearance: appearance ?? null }] : [],
  );
}

export function applyCanvasLayerOrder(
  document: SpaceEditorDocument,
  zIndices: ReadonlyMap<string, number>,
): SpaceEditorDocument {
  return {
    ...document,
    nodes: document.nodes.map((node) => {
      const zIndex = zIndices.get(node.nodeId);
      return zIndex === undefined || zIndex === node.zIndex ? node : { ...node, zIndex };
    }),
  };
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

import type Konva from "konva";
import { nextTick } from "vue";

import type { SpaceEditSnapshot } from "../../../model/structure/schema/edit-snapshot";
import type { CanvasPoint, CanvasRect } from "../../contracts/canvas-editor-port";
import { resizedSectionFrame, type SectionResizeCorner } from "./structure-gesture-preview";

interface SectionFrameGestureInput {
  id: string;
  frame: CanvasRect;
  corner: SectionResizeCorner;
  aspectLocked: boolean;
  target: Konva.Node;
  origin: CanvasPoint;
}

export function createSectionFrameGesture({
  id,
  frame,
  corner,
  aspectLocked,
  target,
  origin,
}: SectionFrameGestureInput) {
  let active = true;
  function position(size: { width: number; height: number }) {
    target.position({
      x: corner.endsWith("west") ? 0 : size.width,
      y: corner.startsWith("north") ? 0 : size.height,
    });
  }
  function command() {
    const point = target.getStage()!.getRelativePointerPosition()!;
    const delta = { x: point.x - origin.x, y: point.y - origin.y };
    const next = resizedSectionFrame(frame, corner, delta, aspectLocked);
    void nextTick(() => {
      if (active) position(next);
    });
    return { kind: "section.resize-frame" as const, structureNodeId: id, frame: next };
  }
  function settle(snapshot: SpaceEditSnapshot) {
    active = false;
    const layout = snapshot.layouts.find(({ structureNodeId }) => structureNodeId === id);
    if (layout?.kind !== "section") throw new Error("Section layout is unavailable");
    position(layout.size);
  }
  return {
    command,
    settle,
    cancel: () => {
      active = false;
    },
  };
}

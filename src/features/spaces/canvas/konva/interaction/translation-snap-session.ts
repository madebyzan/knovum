import {
  canvasFrameBounds,
  translateCanvasBounds,
  unionCanvasBounds,
  type CanvasBounds,
} from "../../canvas-snap-geometry";
import type { CanvasNodeFrame } from "../../schema/document";
import type { CanvasSnapGuide } from "./canvas-snapping";
import type { CanvasTranslationSnapIndex } from "./canvas-translation-snap-index";
import { axisSnap } from "./canvas-translation-snapping";

type Axis = "x" | "y";
interface Lock {
  position: number;
  guides: CanvasSnapGuide[];
}

/** A gesture owns its magnetic targets. Acquire at 6 screen pixels, release at 10, never re-elect while held. */
export function createTranslationSnapSession(
  frames: readonly CanvasNodeFrame[],
  index: CanvasTranslationSnapIndex,
) {
  const original = unionCanvasBounds(frames.map(canvasFrameBounds));
  const lockByAxis: Partial<Record<Axis, Lock>> = {};
  function resolve(moving: CanvasBounds, proposed: { x: number; y: number }, axis: Axis, zoom: number) {
    const held = lockByAxis[axis];
    if (
      held &&
      Math.abs(held.position - proposed[axis]) <= 10 / zoom &&
      intersectsCross(moving, held, axis)
    ) {
      return { position: held.position, guides: held.guides };
    }
    const match = axisSnap(moving, index, axis, 6 / zoom);
    const lock = match ? { position: proposed[axis] + match.offset, guides: match.guides } : undefined;
    if (lock) lockByAxis[axis] = lock;
    else delete lockByAxis[axis];
    return lock ?? { position: proposed[axis], guides: [] };
  }
  return (proposed: { x: number; y: number }, zoom: number) => {
    const moving = translateCanvasBounds(original, proposed);
    const previous = { ...lockByAxis };
    const x = resolve(moving, proposed, "x", zoom);
    const y = resolve(moving, proposed, "y", zoom);
    const placed = translateCanvasBounds(original, { x: x.position, y: y.position });
    for (const axis of ["x", "y"] as const) {
      const lock = lockByAxis[axis];
      if (lock && lock !== previous[axis]) lock.guides = axisSnap(placed, index, axis, 1e-7)?.guides ?? [];
    }
    return {
      x: x.position,
      y: y.position,
      guides: [...(lockByAxis.x?.guides ?? []), ...(lockByAxis.y?.guides ?? [])],
    };
  };
}

function intersectsCross(moving: CanvasBounds, lock: Lock, axis: Axis) {
  const start = axis === "x" ? moving.top : moving.left;
  const end = axis === "x" ? moving.bottom : moving.right;
  return lock.guides.every(({ points, kind }) => {
    if (kind !== "spacing") return true;
    const cross = points[axis === "x" ? 1 : 0];
    return cross >= start && cross <= end;
  });
}

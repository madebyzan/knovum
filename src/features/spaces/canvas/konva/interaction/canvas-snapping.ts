import { canvasFrameBounds } from "../../canvas-snap-geometry";
import type { CanvasNodeFrame } from "../../contract/canvas-document";

export interface CanvasSnapGuide {
  kind: "alignment" | "measurement" | "spacing";
  points: readonly [number, number, number, number];
}

export interface CanvasResizeBadge {
  height: number;
  width: number;
  x: number;
  y: number;
}

export interface CanvasGestureFeedback {
  guides: readonly CanvasSnapGuide[];
  resize?: CanvasResizeBadge;
}

export function nearestCanvasSnap(value: number, targets: readonly number[], tolerance: number) {
  let closest: { distance: number; value: number } | undefined;
  for (const target of targets) {
    const distance = Math.abs(target - value);
    if (distance <= tolerance && (!closest || distance < closest.distance))
      closest = { distance, value: target };
  }
  return closest?.value;
}

export function nearestSortedCanvasSnap(value: number, targets: readonly number[], tolerance: number) {
  let low = 0;
  let high = targets.length;
  while (low < high) {
    const middle = (low + high) >>> 1;
    if ((targets[middle] ?? Number.POSITIVE_INFINITY) < value) low = middle + 1;
    else high = middle;
  }
  const before = targets[low - 1];
  const after = targets[low];
  const closest =
    before === undefined || (after !== undefined && after - value < value - before) ? after : before;
  return closest !== undefined && Math.abs(closest - value) <= tolerance ? closest : undefined;
}

export function frameSnapCoordinates(frames: readonly CanvasNodeFrame[]) {
  return frames.reduce(
    (coordinates, frame) => {
      const bounds = canvasFrameBounds(frame);
      coordinates.x.push(bounds.left, bounds.centerX, bounds.right);
      coordinates.y.push(bounds.top, bounds.centerY, bounds.bottom);
      return coordinates;
    },
    { x: [] as number[], y: [] as number[] },
  );
}

import type Konva from "konva";

import type { CanvasNodeFrame } from "../../contract/canvas-document";
import { frameSnapCoordinates, nearestSortedCanvasSnap } from "./canvas-snapping";

export interface CanvasResizeSnapIndex {
  extent: { bottom: number; left: number; right: number; top: number };
  x: readonly number[];
  y: readonly number[];
}

export function createCanvasResizeSnapIndex(
  frames: readonly CanvasNodeFrame[],
  transform: Konva.Transform,
): CanvasResizeSnapIndex {
  const coordinates = frameSnapCoordinates(frames);
  let left = Number.POSITIVE_INFINITY;
  let top = Number.POSITIVE_INFINITY;
  let right = Number.NEGATIVE_INFINITY;
  let bottom = Number.NEGATIVE_INFINITY;
  for (const frame of frames) {
    left = Math.min(left, frame.x);
    top = Math.min(top, frame.y);
    right = Math.max(right, frame.x + frame.width);
    bottom = Math.max(bottom, frame.y + frame.height);
  }
  return {
    extent: { bottom, left, right, top },
    x: coordinates.x.map((value) => transform.point({ x: value, y: 0 }).x).toSorted((a, b) => a - b),
    y: coordinates.y.map((value) => transform.point({ x: 0, y: value }).y).toSorted((a, b) => a - b),
  };
}

export function resizeSnapCoordinate(options: {
  activeAnchor: string;
  axis: "x" | "y";
  index: CanvasResizeSnapIndex;
  value: number;
}) {
  const enabled =
    options.axis === "x"
      ? options.activeAnchor.includes("left") || options.activeAnchor.includes("right")
      : options.activeAnchor.includes("top") || options.activeAnchor.includes("bottom");
  if (!enabled) return undefined;
  return nearestSortedCanvasSnap(options.value, options.index[options.axis], 6);
}

export function resizeSnapGuides(options: {
  index: CanvasResizeSnapIndex;
  snapped: { x: number; y: number };
  transform: Konva.Transform;
  x: number | undefined;
  y: number | undefined;
}) {
  const { index, snapped, transform, x, y } = options;
  const point = transform.copy().invert().point(snapped);
  const { extent } = index;
  return [
    ...(x === undefined
      ? []
      : [{ kind: "alignment", points: [point.x, extent.top, point.x, extent.bottom] } as const]),
    ...(y === undefined
      ? []
      : [{ kind: "alignment", points: [extent.left, point.y, extent.right, point.y] } as const]),
  ];
}

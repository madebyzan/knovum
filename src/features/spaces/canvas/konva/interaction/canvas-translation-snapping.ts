import {
  boundsEnd,
  boundsPoints,
  boundsSize,
  boundsStart,
  canvasFrameBounds,
  type CanvasBounds,
  crossAxisOverlaps,
  translateCanvasBounds,
  unionCanvasBounds,
} from "../../canvas-snap-geometry";
import type { CanvasNodeFrame } from "../../schema/document";
import { nearestSortedCanvasSnap, type CanvasSnapGuide } from "./canvas-snapping";
import type { CanvasTranslationSnapIndex } from "./canvas-translation-snap-index";

export function snapCanvasTranslation(
  movingFrames: readonly CanvasNodeFrame[],
  index: CanvasTranslationSnapIndex,
  proposed: { x: number; y: number },
  tolerance: number,
) {
  const moving = translateCanvasBounds(unionCanvasBounds(movingFrames.map(canvasFrameBounds)), proposed);
  const horizontal = axisSnap(moving, index, "x", tolerance);
  const vertical = axisSnap(moving, index, "y", tolerance);
  return {
    x: proposed.x + (horizontal?.offset ?? 0),
    y: proposed.y + (vertical?.offset ?? 0),
    guides: [...(horizontal?.guides ?? []), ...(vertical?.guides ?? [])],
  };
}

function axisSnap(
  moving: CanvasBounds,
  index: CanvasTranslationSnapIndex,
  axis: "x" | "y",
  tolerance: number,
) {
  return spacingSnap(moving, index, axis, tolerance) ?? alignmentSnap(moving, index, axis, tolerance);
}

function alignmentSnap(
  moving: CanvasBounds,
  index: CanvasTranslationSnapIndex,
  axis: "x" | "y",
  tolerance: number,
) {
  const targets = axis === "x" ? index.x : index.y;
  const points = boundsPoints(moving, axis);
  const match = nearestAlignment(points, targets, tolerance);
  if (!match) return undefined;
  const moved = translateCanvasBounds(
    moving,
    axis === "x" ? { x: match.offset, y: 0 } : { x: 0, y: match.offset },
  );
  const extent = (axis === "x" ? index.extentByX : index.extentByY).get(match.value);
  const guide: CanvasSnapGuide = {
    kind: "alignment",
    points: alignmentGuidePoints(moved, extent, match.value, axis),
  };
  return { guides: [guide], offset: match.offset };
}

function nearestAlignment(points: readonly number[], targets: readonly number[], tolerance: number) {
  return points.reduce<{ offset: number; value: number } | undefined>((nearest, point) => {
    const target = nearestSortedCanvasSnap(point, targets, tolerance);
    if (target === undefined) return nearest;
    const candidate = { offset: target - point, value: target };
    return !nearest || Math.abs(candidate.offset) < Math.abs(nearest.offset) ? candidate : nearest;
  }, undefined);
}

function alignmentGuidePoints(
  bounds: CanvasBounds,
  extent: readonly [number, number] | undefined,
  value: number,
  axis: "x" | "y",
): [number, number, number, number] {
  return axis === "x"
    ? [
        value,
        Math.min(bounds.top, extent?.[0] ?? bounds.top),
        value,
        Math.max(bounds.bottom, extent?.[1] ?? bounds.bottom),
      ]
    : [
        Math.min(bounds.left, extent?.[0] ?? bounds.left),
        value,
        Math.max(bounds.right, extent?.[1] ?? bounds.right),
        value,
      ];
}

function spacingSnap(
  moving: CanvasBounds,
  index: CanvasTranslationSnapIndex,
  axis: "x" | "y",
  tolerance: number,
) {
  const before = nearestSpacingNeighbor(index, moving, axis, "before");
  const after = nearestSpacingNeighbor(index, moving, axis, "after");
  if (!before || !after) return undefined;
  const available = boundsStart(after, axis) - boundsEnd(before, axis) - boundsSize(moving, axis);
  if (available < 0) return undefined;
  const offset = boundsEnd(before, axis) + available / 2 - boundsStart(moving, axis);
  if (Math.abs(offset) > tolerance) return undefined;
  const moved = translateCanvasBounds(moving, axis === "x" ? { x: offset, y: 0 } : { x: 0, y: offset });
  const cross = axis === "x" ? moved.centerY : moved.centerX;
  return {
    offset,
    guides: [
      spacingGuide(boundsEnd(before, axis), boundsStart(moved, axis), cross, axis),
      spacingGuide(boundsEnd(moved, axis), boundsStart(after, axis), cross, axis),
    ],
  };
}

function nearestSpacingNeighbor(
  index: CanvasTranslationSnapIndex,
  moving: CanvasBounds,
  axis: "x" | "y",
  side: "after" | "before",
) {
  const candidates = index.neighbors[axis][side];
  const step = side === "before" ? -1 : 1;
  for (
    let position = neighborBoundary(candidates, moving, axis, side);
    position >= 0 && position < candidates.length;
    position += step
  ) {
    const target = candidates[position]!;
    if (crossAxisOverlaps(moving, target, axis)) return target;
  }
  return undefined;
}

function neighborBoundary(
  candidates: readonly CanvasBounds[],
  moving: CanvasBounds,
  axis: "x" | "y",
  side: "after" | "before",
) {
  const coordinate = side === "before" ? boundsEnd : boundsStart;
  const boundary = side === "before" ? boundsStart(moving, axis) : boundsEnd(moving, axis);
  let low = 0;
  let high = candidates.length;
  while (low < high) {
    const middle = (low + high) >>> 1;
    const value = coordinate(candidates[middle]!, axis);
    if (value < boundary || (side === "before" && value === boundary)) low = middle + 1;
    else high = middle;
  }
  return side === "before" ? low - 1 : low;
}

function spacingGuide(from: number, to: number, cross: number, axis: "x" | "y"): CanvasSnapGuide {
  return { kind: "spacing", points: axis === "x" ? [from, cross, to, cross] : [cross, from, cross, to] };
}

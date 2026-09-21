import type { CanvasNodeFrame } from "../../contract/canvas-document";
import {
  boundsEnd,
  boundsSize,
  boundsStart,
  canvasFrameBounds,
  type CanvasBounds,
  crossAxisOverlaps,
  translateCanvasBounds,
  unionCanvasBounds,
} from "./canvas-snap-geometry";
import { nearestSortedCanvasSnap, type CanvasSnapGuide } from "./canvas-snapping";
import type { CanvasTranslationSnapIndex, IndexedCanvasBounds } from "./canvas-translation-snap-index";

export { createCanvasTranslationSnapIndex } from "./canvas-translation-snap-index";

export interface CanvasTranslationGestureState {
  frames: ReadonlyMap<string, CanvasNodeFrame>;
  origin: { x: number; y: number };
  primaryId: string;
  snapIndex: CanvasTranslationSnapIndex;
}

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
  return spacingSnap(moving, index.tree, axis, tolerance) ?? alignmentSnap(moving, index, axis, tolerance);
}

function alignmentSnap(
  moving: CanvasBounds,
  index: CanvasTranslationSnapIndex,
  axis: "x" | "y",
  tolerance: number,
) {
  const targets = axis === "x" ? index.x : index.y;
  const points = alignmentPoints(moving, axis);
  const match = nearestAlignment(points, targets, tolerance);
  if (!match) return undefined;
  const moved = translateCanvasBounds(
    moving,
    axis === "x" ? { x: match.offset, y: 0 } : { x: 0, y: match.offset },
  );
  const extent = (axis === "x" ? index.xExtents : index.yExtents).get(match.value);
  const guide: CanvasSnapGuide = {
    kind: "alignment",
    points: alignmentGuidePoints(moved, extent, match.value, axis),
  };
  return { guides: [guide], offset: match.offset };
}

function alignmentPoints(bounds: CanvasBounds, axis: "x" | "y") {
  return axis === "x"
    ? [bounds.left, bounds.centerX, bounds.right]
    : [bounds.top, bounds.centerY, bounds.bottom];
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
  tree: CanvasTranslationSnapIndex["tree"],
  axis: "x" | "y",
  tolerance: number,
) {
  const before = nearestSpacingNeighbor(tree, moving, axis, "before");
  const after = nearestSpacingNeighbor(tree, moving, axis, "after");
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
  tree: CanvasTranslationSnapIndex["tree"],
  moving: CanvasBounds,
  axis: "x" | "y",
  side: "after" | "before",
) {
  const candidates = tree.search(crossAxisSearchBounds(moving, axis));
  let nearest: IndexedCanvasBounds | undefined;
  for (const target of candidates) {
    if (!crossAxisOverlaps(moving, target, axis)) continue;
    const eligible =
      side === "before"
        ? boundsEnd(target, axis) <= boundsStart(moving, axis)
        : boundsStart(target, axis) >= boundsEnd(moving, axis);
    if (!eligible) continue;
    if (!nearest) nearest = target;
    else if (side === "before" && boundsEnd(target, axis) > boundsEnd(nearest, axis)) nearest = target;
    else if (side === "after" && boundsStart(target, axis) < boundsStart(nearest, axis)) nearest = target;
  }
  return nearest;
}

function crossAxisSearchBounds(bounds: CanvasBounds, axis: "x" | "y") {
  return axis === "x"
    ? { minX: -Infinity, maxX: Infinity, minY: bounds.top, maxY: bounds.bottom }
    : { minX: bounds.left, maxX: bounds.right, minY: -Infinity, maxY: Infinity };
}

function spacingGuide(from: number, to: number, cross: number, axis: "x" | "y"): CanvasSnapGuide {
  return { kind: "spacing", points: axis === "x" ? [from, cross, to, cross] : [cross, from, cross, to] };
}

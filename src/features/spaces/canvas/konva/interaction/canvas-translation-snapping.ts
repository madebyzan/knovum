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

type SnapAxis = "x" | "y";

function alignmentSnap(
  moving: CanvasBounds,
  index: CanvasTranslationSnapIndex,
  axis: SnapAxis,
  tolerance: number,
) {
  const match = nearestAlignment(alignmentPoints(moving, axis), axis === "x" ? index.x : index.y, tolerance);
  if (!match) return undefined;
  const moved = translateCanvasBounds(
    moving,
    axis === "x" ? { x: match.offset, y: 0 } : { x: 0, y: match.offset },
  );
  return {
    guides: [alignmentGuide(moved, index, axis, match.value)],
    offset: match.offset,
  };
}

function alignmentPoints(moving: CanvasBounds, axis: SnapAxis) {
  return axis === "x"
    ? [moving.left, moving.centerX, moving.right]
    : [moving.top, moving.centerY, moving.bottom];
}

function nearestAlignment(points: readonly number[], targets: readonly number[], tolerance: number) {
  let match: { offset: number; value: number } | undefined;
  for (const point of points) {
    const target = nearestSortedCanvasSnap(point, targets, tolerance);
    if (target === undefined) continue;
    const candidate = { offset: target - point, value: target };
    if (!match || Math.abs(candidate.offset) < Math.abs(match.offset)) match = candidate;
  }
  return match;
}

function alignmentGuide(
  moved: CanvasBounds,
  index: CanvasTranslationSnapIndex,
  axis: SnapAxis,
  value: number,
): CanvasSnapGuide {
  return axis === "x"
    ? verticalAlignmentGuide(moved, index.xExtents.get(value), value)
    : horizontalAlignmentGuide(moved, index.yExtents.get(value), value);
}

function verticalAlignmentGuide(
  moved: CanvasBounds,
  extent: readonly [number, number] | undefined,
  value: number,
): CanvasSnapGuide {
  return {
    kind: "alignment",
    points: [
      value,
      Math.min(moved.top, extent?.[0] ?? moved.top),
      value,
      Math.max(moved.bottom, extent?.[1] ?? moved.bottom),
    ],
  };
}

function horizontalAlignmentGuide(
  moved: CanvasBounds,
  extent: readonly [number, number] | undefined,
  value: number,
): CanvasSnapGuide {
  return {
    kind: "alignment",
    points: [
      Math.min(moved.left, extent?.[0] ?? moved.left),
      value,
      Math.max(moved.right, extent?.[1] ?? moved.right),
      value,
    ],
  };
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

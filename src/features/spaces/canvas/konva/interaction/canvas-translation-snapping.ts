import {
  boundsPoints,
  canvasFrameBounds,
  type CanvasBounds,
  translateCanvasBounds,
  unionCanvasBounds,
} from "../../canvas-snap-geometry";
import type { CanvasNodeFrame } from "../../schema/document";
import { nearestSortedCanvasSnap, type CanvasSnapGuide } from "./canvas-snapping";
import type { CanvasTranslationSnapIndex } from "./canvas-translation-snap-index";
import { spacingSnap } from "./spacing-snap";

export function snapCanvasTranslation(
  movingFrames: readonly CanvasNodeFrame[],
  index: CanvasTranslationSnapIndex,
  proposed: { x: number; y: number },
  tolerance: number,
) {
  const moving = translateCanvasBounds(unionCanvasBounds(movingFrames.map(canvasFrameBounds)), proposed);
  const horizontal = axisSnap(moving, index, "x", tolerance);
  const vertical = axisSnap(moving, index, "y", tolerance);
  const offset = { x: horizontal?.offset ?? 0, y: vertical?.offset ?? 0 };
  const placed = translateCanvasBounds(moving, offset);
  // Feedback describes the final two-axis placement, not the intermediate horizontal/vertical proposals.
  const guidesX = axisSnap(placed, index, "x", 1e-7)?.guides ?? [];
  const guidesY = axisSnap(placed, index, "y", 1e-7)?.guides ?? [];
  return { x: proposed.x + offset.x, y: proposed.y + offset.y, guides: [...guidesX, ...guidesY] };
}

function axisSnap(
  moving: CanvasBounds,
  index: CanvasTranslationSnapIndex,
  axis: "x" | "y",
  tolerance: number,
) {
  const spacing = spacingSnap(moving, index, axis, tolerance);
  const alignment = alignmentSnap(moving, index, axis, tolerance);
  if (!spacing) return alignment;
  if (!alignment) return spacing;
  return Math.abs(alignment.offset) < Math.abs(spacing.offset) ? alignment : spacing;
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

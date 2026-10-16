import {
  boundsEnd,
  boundsSize,
  boundsStart,
  translateCanvasBounds,
  type CanvasBounds,
} from "../../canvas-snap-geometry";
import type { CanvasSnapGuide } from "./canvas-snapping";
import type { CanvasTranslationSnapIndex } from "./canvas-translation-snap-index";
import { nearestSpacingNeighbor } from "./spacing-neighbor";

type Axis = "x" | "y";
interface SpacingMatch {
  offset: number;
  guides: CanvasSnapGuide[];
}

function spacingGuide(from: number, to: number, cross: number, axis: Axis): CanvasSnapGuide {
  return { kind: "spacing", points: axis === "x" ? [from, cross, to, cross] : [cross, from, cross, to] };
}

function commonCross(bounds: readonly CanvasBounds[], axis: Axis) {
  const start = Math.max(...bounds.map((box) => (axis === "x" ? box.top : box.left)));
  const end = Math.min(...bounds.map((box) => (axis === "x" ? box.bottom : box.right)));
  return start < end ? (start + end) / 2 : undefined;
}

/** Equal gaps between neighbors, or continuation of a neighboring pair's existing gap. */
export function spacingSnap(
  moving: CanvasBounds,
  index: CanvasTranslationSnapIndex,
  axis: Axis,
  tolerance: number,
) {
  const before = nearestSpacingNeighbor(index, moving, axis, "before");
  const after = nearestSpacingNeighbor(index, moving, axis, "after");
  const candidates: SpacingMatch[] = [];
  const centered = centeredSpacing({ moving, before, after, axis });
  if (centered) candidates.push(centered);
  for (const side of ["before", "after"] as const) {
    const neighbor = side === "before" ? before : after;
    if (!neighbor) continue;
    const continuation = continueSpacing({ moving, neighbor, axis, side }, index);
    if (continuation) candidates.push(continuation);
  }
  return candidates
    .filter(({ offset }) => Math.abs(offset) <= tolerance)
    .toSorted((left, right) => Math.abs(left.offset) - Math.abs(right.offset))[0];
}

function centeredSpacing(input: {
  moving: CanvasBounds;
  before: CanvasBounds | undefined;
  after: CanvasBounds | undefined;
  axis: Axis;
}): SpacingMatch | undefined {
  const { moving, before, after, axis } = input;
  if (!before || !after) return undefined;
  const cross = commonCross([before, moving, after], axis);
  const gap = (boundsStart(after, axis) - boundsEnd(before, axis) - boundsSize(moving, axis)) / 2;
  if (cross === undefined || gap <= 0) return undefined;
  const offset = boundsEnd(before, axis) + gap - boundsStart(moving, axis);
  return {
    offset,
    guides: [
      spacingGuide(boundsEnd(before, axis), boundsStart(moving, axis) + offset, cross, axis),
      spacingGuide(boundsEnd(moving, axis) + offset, boundsStart(after, axis), cross, axis),
    ],
  };
}

function continueSpacing(
  input: { moving: CanvasBounds; neighbor: CanvasBounds; axis: Axis; side: "before" | "after" },
  index: CanvasTranslationSnapIndex,
): SpacingMatch | undefined {
  const { moving, neighbor, axis, side } = input;
  const outer = nearestSpacingNeighbor(index, neighbor, axis, side);
  if (!outer) return undefined;
  const earlier = side === "before" ? outer : neighbor;
  const later = side === "before" ? neighbor : outer;
  const gap = boundsStart(later, axis) - boundsEnd(earlier, axis);
  if (gap <= 0) return undefined;
  const offset =
    side === "before"
      ? boundsEnd(neighbor, axis) + gap - boundsStart(moving, axis)
      : boundsStart(neighbor, axis) - gap - boundsEnd(moving, axis);
  const moved = translateCanvasBounds(moving, axis === "x" ? { x: offset, y: 0 } : { x: 0, y: offset });
  const cross = commonCross([earlier, later, moved], axis);
  if (cross === undefined) return undefined;
  return {
    offset,
    guides: [
      spacingGuide(boundsEnd(earlier, axis), boundsStart(later, axis), cross, axis),
      spacingGuide(
        side === "before" ? boundsEnd(neighbor, axis) : boundsEnd(moved, axis),
        side === "before" ? boundsStart(moved, axis) : boundsStart(neighbor, axis),
        cross,
        axis,
      ),
    ],
  };
}

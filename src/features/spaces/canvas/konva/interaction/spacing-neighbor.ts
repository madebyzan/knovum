import { boundsEnd, boundsStart, crossAxisOverlaps, type CanvasBounds } from "../../canvas-snap-geometry";
import type { CanvasTranslationSnapIndex } from "./canvas-translation-snap-index";

export function nearestSpacingNeighbor(
  index: CanvasTranslationSnapIndex,
  moving: CanvasBounds,
  axis: "x" | "y",
  side: "after" | "before",
) {
  const candidates = index.neighborsByAxis[axis][side];
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

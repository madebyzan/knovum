import { canvasFrameBounds, type CanvasBounds } from "../../canvas-snap-geometry";
import type { CanvasNodeFrame } from "../../schema/document";

export interface CanvasTranslationSnapIndex {
  neighbors: Readonly<Record<"x" | "y", { before: readonly CanvasBounds[]; after: readonly CanvasBounds[] }>>;
  x: readonly number[];
  extentByX: ReadonlyMap<number, readonly [number, number]>;
  y: readonly number[];
  extentByY: ReadonlyMap<number, readonly [number, number]>;
}

export function createCanvasTranslationSnapIndex(frames: readonly CanvasNodeFrame[]) {
  const bounds = frames.map(canvasFrameBounds);
  return {
    neighbors: {
      x: {
        before: bounds.toSorted((a, b) => a.right - b.right),
        after: bounds.toSorted((a, b) => a.left - b.left),
      },
      y: {
        before: bounds.toSorted((a, b) => a.bottom - b.bottom),
        after: bounds.toSorted((a, b) => a.top - b.top),
      },
    },
    x: bounds.flatMap(({ left, centerX, right }) => [left, centerX, right]).toSorted(ascending),
    extentByX: alignmentExtents(bounds, "x"),
    y: bounds.flatMap(({ top, centerY, bottom }) => [top, centerY, bottom]).toSorted(ascending),
    extentByY: alignmentExtents(bounds, "y"),
  } satisfies CanvasTranslationSnapIndex;
}

function alignmentExtents(bounds: readonly CanvasBounds[], axis: "x" | "y") {
  const extentByCoordinate = new Map<number, readonly [number, number]>();
  for (const box of bounds) {
    const coordinates =
      axis === "x" ? [box.left, box.centerX, box.right] : [box.top, box.centerY, box.bottom];
    const cross: readonly [number, number] = axis === "x" ? [box.top, box.bottom] : [box.left, box.right];
    for (const coordinate of coordinates) {
      const current = extentByCoordinate.get(coordinate);
      extentByCoordinate.set(
        coordinate,
        current ? [Math.min(current[0], cross[0]), Math.max(current[1], cross[1])] : cross,
      );
    }
  }
  return extentByCoordinate;
}

const ascending = (left: number, right: number) => left - right;

import RBush from "rbush";

import { canvasFrameBounds, type CanvasBounds } from "../../canvas-snap-geometry";
import type { CanvasNodeFrame } from "../../schema/document";

export interface IndexedCanvasBounds extends CanvasBounds {
  maxX: number;
  maxY: number;
  minX: number;
  minY: number;
}

export interface CanvasTranslationSnapIndex {
  tree: RBush<IndexedCanvasBounds>;
  x: readonly number[];
  xExtents: ReadonlyMap<number, readonly [number, number]>;
  y: readonly number[];
  yExtents: ReadonlyMap<number, readonly [number, number]>;
}

export function createCanvasTranslationSnapIndex(frames: readonly CanvasNodeFrame[]) {
  const bounds = frames.map(canvasFrameBounds);
  return {
    tree: new RBush<IndexedCanvasBounds>().load(
      bounds.map((box) => ({
        ...box,
        minX: box.left,
        minY: box.top,
        maxX: box.right,
        maxY: box.bottom,
      })),
    ),
    x: bounds.flatMap(({ left, centerX, right }) => [left, centerX, right]).toSorted(ascending),
    xExtents: alignmentExtents(bounds, "x"),
    y: bounds.flatMap(({ top, centerY, bottom }) => [top, centerY, bottom]).toSorted(ascending),
    yExtents: alignmentExtents(bounds, "y"),
  } satisfies CanvasTranslationSnapIndex;
}

function alignmentExtents(bounds: readonly CanvasBounds[], axis: "x" | "y") {
  const extents = new Map<number, readonly [number, number]>();
  for (const box of bounds) {
    const coordinates =
      axis === "x" ? [box.left, box.centerX, box.right] : [box.top, box.centerY, box.bottom];
    const cross: readonly [number, number] = axis === "x" ? [box.top, box.bottom] : [box.left, box.right];
    for (const coordinate of coordinates) {
      const current = extents.get(coordinate);
      extents.set(
        coordinate,
        current ? [Math.min(current[0], cross[0]), Math.max(current[1], cross[1])] : cross,
      );
    }
  }
  return extents;
}

const ascending = (left: number, right: number) => left - right;

import type { CanvasNodeFrame } from "../../contract/canvas-document";

export interface CanvasBounds {
  bottom: number;
  centerX: number;
  centerY: number;
  height: number;
  left: number;
  right: number;
  top: number;
  width: number;
}

export const boundsPoints = (bounds: CanvasBounds, axis: "x" | "y") =>
  axis === "x" ? [bounds.left, bounds.centerX, bounds.right] : [bounds.top, bounds.centerY, bounds.bottom];
export const boundsStart = (bounds: CanvasBounds, axis: "x" | "y") =>
  axis === "x" ? bounds.left : bounds.top;
export const boundsEnd = (bounds: CanvasBounds, axis: "x" | "y") =>
  axis === "x" ? bounds.right : bounds.bottom;
export const boundsSize = (bounds: CanvasBounds, axis: "x" | "y") =>
  axis === "x" ? bounds.width : bounds.height;

export function crossAxisOverlaps(moving: CanvasBounds, target: CanvasBounds, axis: "x" | "y") {
  return axis === "x"
    ? moving.bottom >= target.top && moving.top <= target.bottom
    : moving.right >= target.left && moving.left <= target.right;
}

export function unionCanvasBounds(bounds: readonly CanvasBounds[]): CanvasBounds {
  return boundsFromEdges(
    Math.min(...bounds.map(({ left }) => left)),
    Math.min(...bounds.map(({ top }) => top)),
    Math.max(...bounds.map(({ right }) => right)),
    Math.max(...bounds.map(({ bottom }) => bottom)),
  );
}

export function canvasFrameBounds(frame: CanvasNodeFrame): CanvasBounds {
  const radians = (frame.rotation * Math.PI) / 180;
  const cosine = Math.cos(radians);
  const sine = Math.sin(radians);
  const corners = [
    [0, 0],
    [frame.width, 0],
    [0, frame.height],
    [frame.width, frame.height],
  ].map(([x = 0, y = 0]) => ({
    x: frame.x + x * cosine - y * sine,
    y: frame.y + x * sine + y * cosine,
  }));
  return boundsFromEdges(
    Math.min(...corners.map(({ x }) => x)),
    Math.min(...corners.map(({ y }) => y)),
    Math.max(...corners.map(({ x }) => x)),
    Math.max(...corners.map(({ y }) => y)),
  );
}

export function translateCanvasBounds(bounds: CanvasBounds, offset: { x: number; y: number }) {
  return boundsFromEdges(
    bounds.left + offset.x,
    bounds.top + offset.y,
    bounds.right + offset.x,
    bounds.bottom + offset.y,
  );
}

function boundsFromEdges(left: number, top: number, right: number, bottom: number): CanvasBounds {
  return {
    bottom,
    centerX: (left + right) / 2,
    centerY: (top + bottom) / 2,
    height: bottom - top,
    left,
    right,
    top,
    width: right - left,
  };
}

import { clamp } from "es-toolkit";

import type { CanvasPoint } from "../../contracts/canvas-editor";

const EDGE_ZONE = 56;
const MAX_SPEED = 720;

export function edgeAutoPanVelocity(point: CanvasPoint, size: { width: number; height: number }) {
  return {
    x: axisVelocity(point.x, size.width),
    y: axisVelocity(point.y, size.height),
  };
}

function axisVelocity(position: number, extent: number) {
  if (position < EDGE_ZONE) return -MAX_SPEED * intensity(EDGE_ZONE - position);
  if (position > extent - EDGE_ZONE) return MAX_SPEED * intensity(position - (extent - EDGE_ZONE));
  return 0;
}

function intensity(distance: number) {
  const normalized = clamp(distance / EDGE_ZONE, 0, 1);
  return normalized * normalized;
}

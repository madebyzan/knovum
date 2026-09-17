import type { CanvasPoint } from "../../contract/canvas-editor";

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
  const normalized = Math.min(1, Math.max(0, distance / EDGE_ZONE));
  return normalized * normalized;
}

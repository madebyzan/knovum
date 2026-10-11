import type { CanvasPoint } from "../../contracts/canvas-editor-port";

export interface GroupHandleOrigin {
  point: CanvasPoint;
  center: CanvasPoint;
}

export interface GroupHandleTransform {
  anchor: CanvasPoint;
  rotationDelta: number;
  scaleFactor: number;
}

export function groupHandleTransform(
  kind: "rotate" | "scale",
  origin: GroupHandleOrigin,
  point: CanvasPoint,
): GroupHandleTransform {
  const initialVector = { x: origin.point.x - origin.center.x, y: origin.point.y - origin.center.y };
  const nextVector = { x: point.x - origin.center.x, y: point.y - origin.center.y };
  const initialDistance = Math.hypot(initialVector.x, initialVector.y);
  return {
    anchor: origin.center,
    rotationDelta:
      kind === "rotate"
        ? ((Math.atan2(nextVector.y, nextVector.x) - Math.atan2(initialVector.y, initialVector.x)) * 180) /
          Math.PI
        : 0,
    scaleFactor:
      kind === "scale" && initialDistance
        ? Math.max(0.01, Math.hypot(nextVector.x, nextVector.y) / initialDistance)
        : 1,
  };
}

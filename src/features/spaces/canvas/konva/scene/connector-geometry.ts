import type { CanvasNodeLayoutV1 } from "../../contract/canvas-document";
import type { CanvasPoint } from "../../contract/canvas-editor";

export type CanvasConnectorSide = "top" | "right" | "bottom" | "left";

export function connectorAnchor(frame: CanvasNodeLayoutV1["frame"], side: CanvasConnectorSide) {
  const local =
    side === "top"
      ? { x: frame.width / 2, y: 0 }
      : side === "bottom"
        ? { x: frame.width / 2, y: frame.height }
        : side === "left"
          ? { x: 0, y: frame.height / 2 }
          : { x: frame.width, y: frame.height / 2 };
  return fromLocal(frame, local);
}

export function connectorControlCenter(
  frame: CanvasNodeLayoutV1["frame"],
  side: CanvasConnectorSide,
  distance: number,
) {
  const anchor = connectorAnchor(frame, side);
  const angle = ((frame.rotation + sideRotation(side)) * Math.PI) / 180;
  return { x: anchor.x + Math.cos(angle) * distance, y: anchor.y + Math.sin(angle) * distance };
}

export function frameContainsPoint(frame: CanvasNodeLayoutV1["frame"], point: CanvasPoint) {
  const local = toLocal(frame, point);
  return local.x >= 0 && local.x <= frame.width && local.y >= 0 && local.y <= frame.height;
}

export function nearestConnectorSide(
  frame: CanvasNodeLayoutV1["frame"],
  point: CanvasPoint,
): CanvasConnectorSide {
  const local = toLocal(frame, point);
  const distances = [
    ["top", Math.abs(local.y)],
    ["right", Math.abs(local.x - frame.width)],
    ["bottom", Math.abs(local.y - frame.height)],
    ["left", Math.abs(local.x)],
  ] as const;
  return distances.reduce((nearest, candidate) => (candidate[1] < nearest[1] ? candidate : nearest))[0];
}

export function nearestConnectorTarget(
  layouts: readonly CanvasNodeLayoutV1[],
  point: CanvasPoint,
  maximumDistance: number,
) {
  let nearest:
    | { nodeId: string; side: CanvasConnectorSide; anchor: CanvasPoint; distance: number }
    | undefined;
  for (const { nodeId, frame } of layouts) {
    for (const side of ["top", "right", "bottom", "left"] as const) {
      const anchor = connectorAnchor(frame, side);
      const distance = Math.hypot(anchor.x - point.x, anchor.y - point.y);
      if (distance <= maximumDistance && (!nearest || distance < nearest.distance)) {
        nearest = { nodeId, side, anchor, distance };
      }
    }
  }
  return nearest && { nodeId: nearest.nodeId, side: nearest.side, anchor: nearest.anchor };
}

function toLocal(frame: CanvasNodeLayoutV1["frame"], point: CanvasPoint) {
  const angle = (-frame.rotation * Math.PI) / 180;
  const x = point.x - frame.x;
  const y = point.y - frame.y;
  return { x: x * Math.cos(angle) - y * Math.sin(angle), y: x * Math.sin(angle) + y * Math.cos(angle) };
}

function fromLocal(frame: CanvasNodeLayoutV1["frame"], point: CanvasPoint) {
  const angle = (frame.rotation * Math.PI) / 180;
  return {
    x: frame.x + point.x * Math.cos(angle) - point.y * Math.sin(angle),
    y: frame.y + point.x * Math.sin(angle) + point.y * Math.cos(angle),
  };
}

function sideRotation(side: CanvasConnectorSide) {
  if (side === "right") return 0;
  if (side === "bottom") return 90;
  if (side === "left") return 180;
  return 270;
}

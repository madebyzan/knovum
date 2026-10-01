import { clamp } from "es-toolkit";

import type { CanvasPoint, CanvasRect } from "../../contracts/canvas-editor-port";
import type { CanvasNodeFrame } from "../../schema/document";
import { connectorAnchor, type CanvasConnectorSide } from "./connector-geometry";

export type CanvasRelationPath = readonly [number, number, number, number, number, number, number, number];

export interface CanvasRelationGeometry {
  bounds: CanvasRect;
  fromSide: CanvasConnectorSide;
  midpoint: CanvasPoint;
  points: CanvasRelationPath;
  toSide: CanvasConnectorSide;
}

export function relationGeometry(
  source: CanvasNodeFrame,
  target: CanvasNodeFrame,
  parallelOffset = 0,
): CanvasRelationGeometry {
  const fromSide = inferredSide(source, target);
  const toSide = inferredSide(target, source);
  const from = connectorAnchor(source, fromSide);
  const to = connectorAnchor(target, toSide);
  const points = offsetBezierControls(bezierConnectorPoints(from, fromSide, to, toSide), parallelOffset);
  return {
    bounds: pathBounds(points),
    fromSide,
    midpoint: cubicPoint(points, 0.5),
    points,
    toSide,
  };
}

function offsetBezierControls(points: CanvasRelationPath, offset: number): CanvasRelationPath {
  if (offset === 0) return points;
  const [x0, y0, x1, y1, x2, y2, x3, y3] = points;
  const length = Math.hypot(x3 - x0, y3 - y0);
  if (length === 0) return points;
  const offsetX = (-(y3 - y0) / length) * offset;
  const offsetY = ((x3 - x0) / length) * offset;
  return [x0, y0, x1 + offsetX, y1 + offsetY, x2 + offsetX, y2 + offsetY, x3, y3];
}

interface DraftRelationPathInput {
  from: CanvasPoint;
  fromSide: CanvasConnectorSide;
  to: CanvasPoint;
  toSide?: CanvasConnectorSide | undefined;
}

export function draftRelationPath({ from, fromSide, to, toSide }: DraftRelationPathInput) {
  return bezierConnectorPoints(from, fromSide, to, toSide ?? oppositeSide(fromSide));
}

export function cubicPoint(points: CanvasRelationPath, t: number): CanvasPoint {
  const inverse = 1 - t;
  const [x0, y0, x1, y1, x2, y2, x3, y3] = points;
  return {
    x: inverse ** 3 * x0 + 3 * inverse ** 2 * t * x1 + 3 * inverse * t ** 2 * x2 + t ** 3 * x3,
    y: inverse ** 3 * y0 + 3 * inverse ** 2 * t * y1 + 3 * inverse * t ** 2 * y2 + t ** 3 * y3,
  };
}

function bezierConnectorPoints(
  from: CanvasPoint,
  fromSide: CanvasConnectorSide,
  to: CanvasPoint,
  toSide: CanvasConnectorSide,
): CanvasRelationPath {
  const distance = Math.hypot(to.x - from.x, to.y - from.y);
  const tangent = clamp(distance * 0.42, 56, 220);
  const first = moveAlongSide(from, fromSide, tangent);
  const second = moveAlongSide(to, toSide, tangent);
  return [from.x, from.y, first.x, first.y, second.x, second.y, to.x, to.y];
}

function inferredSide(frame: CanvasNodeFrame, other: CanvasNodeFrame): CanvasConnectorSide {
  const center = { x: frame.x + frame.width / 2, y: frame.y + frame.height / 2 };
  const target = { x: other.x + other.width / 2, y: other.y + other.height / 2 };
  if (Math.abs(target.x - center.x) >= Math.abs(target.y - center.y)) {
    return target.x >= center.x ? "right" : "left";
  }
  return target.y >= center.y ? "bottom" : "top";
}

function moveAlongSide(point: CanvasPoint, side: CanvasConnectorSide, distance: number): CanvasPoint {
  if (side === "left") return { x: point.x - distance, y: point.y };
  if (side === "right") return { x: point.x + distance, y: point.y };
  if (side === "top") return { x: point.x, y: point.y - distance };
  return { x: point.x, y: point.y + distance };
}

function oppositeSide(side: CanvasConnectorSide): CanvasConnectorSide {
  if (side === "left") return "right";
  if (side === "right") return "left";
  if (side === "top") return "bottom";
  return "top";
}

function pathBounds(points: CanvasRelationPath): CanvasRect {
  const parameters = new Set([
    0,
    1,
    ...cubicExtrema(points[0], points[2], points[4], points[6]),
    ...cubicExtrema(points[1], points[3], points[5], points[7]),
  ]);
  const samples = [...parameters].map((parameter) => cubicPoint(points, parameter));
  const xs = samples.map(({ x }) => x);
  const ys = samples.map(({ y }) => y);
  const x = Math.min(...xs);
  const y = Math.min(...ys);
  return { x, y, width: Math.max(...xs) - x, height: Math.max(...ys) - y };
}

function cubicExtrema(start: number, controlA: number, controlB: number, end: number) {
  const a = -start + 3 * controlA - 3 * controlB + end;
  const b = 2 * (start - 2 * controlA + controlB);
  const c = controlA - start;
  if (Math.abs(a) < Number.EPSILON) return Math.abs(b) < Number.EPSILON ? [] : validParameters([-c / b]);
  const discriminant = b * b - 4 * a * c;
  if (discriminant < 0) return [];
  const root = Math.sqrt(discriminant);
  return validParameters([(-b + root) / (2 * a), (-b - root) / (2 * a)]);
}

function validParameters(parameters: readonly number[]) {
  return parameters.filter((parameter) => parameter > 0 && parameter < 1);
}

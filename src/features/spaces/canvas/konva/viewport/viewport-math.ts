import { clamp } from "es-toolkit";

import type { CanvasPoint, CanvasViewport } from "../../contracts/canvas-editor";

export const MIN_CANVAS_ZOOM = 0.1;
export const MAX_CANVAS_ZOOM = 4;

export function canvasPointFromScreen(point: CanvasPoint, viewport: CanvasViewport): CanvasPoint {
  return {
    x: (point.x - viewport.x) / viewport.zoom,
    y: (point.y - viewport.y) / viewport.zoom,
  };
}

export function screenPointFromCanvas(point: CanvasPoint, viewport: CanvasViewport): CanvasPoint {
  return {
    x: point.x * viewport.zoom + viewport.x,
    y: point.y * viewport.zoom + viewport.y,
  };
}

export function zoomViewportAt(
  viewport: CanvasViewport,
  screenPoint: CanvasPoint,
  requestedZoom: number,
): CanvasViewport {
  const zoom = clamp(requestedZoom, MIN_CANVAS_ZOOM, MAX_CANVAS_ZOOM);
  const canvasPoint = canvasPointFromScreen(screenPoint, viewport);
  return {
    x: screenPoint.x - canvasPoint.x * zoom,
    y: screenPoint.y - canvasPoint.y * zoom,
    zoom,
  };
}

export function normalizedWheelDelta(event: WheelEvent, pageHeight: number) {
  const factor =
    event.deltaMode === WheelEvent.DOM_DELTA_LINE
      ? 16
      : event.deltaMode === WheelEvent.DOM_DELTA_PAGE
        ? pageHeight
        : 1;
  return { x: event.deltaX * factor, y: event.deltaY * factor };
}

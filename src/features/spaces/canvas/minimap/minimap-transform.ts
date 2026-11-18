import { clamp } from "es-toolkit";

import type { WorldBounds } from "../../model/structure/world-bounds";
import type { CanvasPoint, CanvasViewport } from "../contracts/canvas-editor-port";
import { boundsCenter } from "./minimap-scene";

export const MINIMAP_DIMENSIONS = {
  width: 184,
  height: 112,
  padding: 8,
} as const;

const MIN_MAP_WORLD_WIDTH = 1280;
const MIN_MAP_WORLD_HEIGHT = 800;
const WORLD_MARGIN_RATIO = 0.12;
const MIN_WORLD_MARGIN = 96;
const MIN_VIEWPORT_BOX_PX = 8;

export interface MinimapDimensions {
  readonly width: number;
  readonly height: number;
  readonly padding: number;
}

export interface MinimapTransform {
  readonly scale: number;
  readonly offsetX: number;
  readonly offsetY: number;
  readonly worldBounds: WorldBounds;
}

export interface MinimapRect {
  readonly left: number;
  readonly top: number;
  readonly width: number;
  readonly height: number;
}

export interface MinimapViewportProjection {
  readonly rect: MinimapRect;
  readonly intersectsMap: boolean;
  readonly offscreenIndicator?: {
    readonly x: number;
    readonly y: number;
    readonly angleDeg: number;
  };
}

/**
 * Computes a stable world-to-minimap transform anchored solely to content bounds so panning or
 * zooming the camera never rescales or jitters the minimap world.
 */
export function computeMinimapTransform(
  contentBounds: WorldBounds,
  dimensions: MinimapDimensions = MINIMAP_DIMENSIONS,
): MinimapTransform {
  const rawWidth = Math.max(1, contentBounds.right - contentBounds.left);
  const rawHeight = Math.max(1, contentBounds.bottom - contentBounds.top);
  const margin = Math.max(MIN_WORLD_MARGIN, Math.max(rawWidth, rawHeight) * WORLD_MARGIN_RATIO);
  const spanWidth = Math.max(MIN_MAP_WORLD_WIDTH, rawWidth + margin * 2);
  const spanHeight = Math.max(MIN_MAP_WORLD_HEIGHT, rawHeight + margin * 2);
  const center = boundsCenter(contentBounds);
  const worldBounds: WorldBounds = {
    left: center.x - spanWidth / 2,
    top: center.y - spanHeight / 2,
    right: center.x + spanWidth / 2,
    bottom: center.y + spanHeight / 2,
  };
  const innerWidth = Math.max(1, dimensions.width - dimensions.padding * 2);
  const innerHeight = Math.max(1, dimensions.height - dimensions.padding * 2);
  const scale = Math.min(innerWidth / spanWidth, innerHeight / spanHeight);
  return {
    scale,
    offsetX: dimensions.width / 2 - center.x * scale,
    offsetY: dimensions.height / 2 - center.y * scale,
    worldBounds,
  };
}

/** Projects the active camera viewport into minimap pixel coordinates, plus an offscreen beacon if needed. */
export function computeMinimapViewport(
  viewport: CanvasViewport,
  hostSize: { readonly width: number; readonly height: number },
  transform: MinimapTransform,
  dimensions: MinimapDimensions = MINIMAP_DIMENSIONS,
): MinimapViewportProjection {
  const rawLeft = (-viewport.x / viewport.zoom) * transform.scale + transform.offsetX;
  const rawTop = (-viewport.y / viewport.zoom) * transform.scale + transform.offsetY;
  const rawWidth = (hostSize.width / viewport.zoom) * transform.scale;
  const rawHeight = (hostSize.height / viewport.zoom) * transform.scale;
  const width = Math.max(MIN_VIEWPORT_BOX_PX, rawWidth);
  const height = Math.max(MIN_VIEWPORT_BOX_PX, rawHeight);
  const left = rawLeft - (width - rawWidth) / 2;
  const top = rawTop - (height - rawHeight) / 2;
  const intersectsMap =
    left + width > 0 && top + height > 0 && left < dimensions.width && top < dimensions.height;
  const rect = { left, top, width, height };
  if (intersectsMap) return { rect, intersectsMap };
  const centerX = rawLeft + rawWidth / 2;
  const centerY = rawTop + rawHeight / 2;
  const pad = 7;
  const angleDeg =
    (Math.atan2(centerY - dimensions.height / 2, centerX - dimensions.width / 2) * 180) / Math.PI;
  return {
    rect,
    intersectsMap,
    offscreenIndicator: {
      x: clamp(centerX, pad, dimensions.width - pad),
      y: clamp(centerY, pad, dimensions.height - pad),
      angleDeg,
    },
  };
}

export function isPointInMinimapRect(point: CanvasPoint, rect: MinimapRect, hitSlopPx = 2): boolean {
  return (
    point.x >= rect.left - hitSlopPx &&
    point.x <= rect.left + rect.width + hitSlopPx &&
    point.y >= rect.top - hitSlopPx &&
    point.y <= rect.top + rect.height + hitSlopPx
  );
}

export function minimapPointToWorld(point: CanvasPoint, transform: MinimapTransform): CanvasPoint {
  return {
    x: (point.x - transform.offsetX) / transform.scale,
    y: (point.y - transform.offsetY) / transform.scale,
  };
}

export function viewportCenteredAtWorld(
  worldPoint: CanvasPoint,
  zoom: number,
  hostSize: { readonly width: number; readonly height: number },
): CanvasViewport {
  return {
    x: hostSize.width / 2 - worldPoint.x * zoom,
    y: hostSize.height / 2 - worldPoint.y * zoom,
    zoom,
  };
}

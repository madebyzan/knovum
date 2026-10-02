import { canvasFrameBounds } from "../../canvas-snap-geometry";
import type { CanvasViewport } from "../../contracts/canvas-editor-port";
import type { CanvasScene } from "../scene/canvas-scene";

export interface CanvasOverviewViewport {
  x: number;
  y: number;
  scale: number;
}

export function canvasViewportThumbnail(
  viewport: CanvasViewport,
  source: { width: number; height: number },
  target: { width: number; height: number },
): CanvasOverviewViewport {
  if (source.width <= 0 || source.height <= 0) throw new Error("Canvas viewport has no drawable area");
  const scale = Math.min(target.width / source.width, target.height / source.height);
  return {
    x: (target.width - source.width * scale) / 2 + viewport.x * scale,
    y: (target.height - source.height * scale) / 2 + viewport.y * scale,
    scale: viewport.zoom * scale,
  };
}

export function canvasOverviewViewport(
  scene: CanvasScene,
  width: number,
  height: number,
): CanvasOverviewViewport {
  let left = Infinity;
  let top = Infinity;
  let right = -Infinity;
  let bottom = -Infinity;
  for (const { layout } of scene.nodes) {
    const bounds = canvasFrameBounds(layout.frame);
    include({ x: bounds.left, y: bounds.top });
    include({ x: bounds.right, y: bounds.bottom });
  }
  for (const { bounds } of scene.sectionBackdrops) {
    include({ x: bounds.x, y: bounds.y });
    include({ x: bounds.x + bounds.width, y: bounds.y + bounds.height });
  }
  for (const { bounds } of scene.relations) {
    include({ x: bounds.x, y: bounds.y });
    include({ x: bounds.x + bounds.width, y: bounds.y + bounds.height });
  }
  if (left === Infinity) return { x: 0, y: 0, scale: 1 };

  const contentWidth = Math.max(1, right - left);
  const contentHeight = Math.max(1, bottom - top);
  const margin = 20;
  const scale = Math.min((width - margin * 2) / contentWidth, (height - margin * 2) / contentHeight, 1);
  return {
    x: (width - contentWidth * scale) / 2 - left * scale,
    y: (height - contentHeight * scale) / 2 - top * scale,
    scale,
  };

  function include(point: { x: number; y: number }) {
    left = Math.min(left, point.x);
    top = Math.min(top, point.y);
    right = Math.max(right, point.x);
    bottom = Math.max(bottom, point.y);
  }
}

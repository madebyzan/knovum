import { compact, intersectionBy } from "es-toolkit";

import type { CanvasViewport } from "../../contract/canvas-editor-port";
import type { CanvasNodeLayoutV1 } from "../../schema/document";
import { screenPointFromCanvas, zoomViewportAt } from "./viewport-math";

/** Canvas layout nodeIds are schema-unique, so the intersection cannot drop a layout. */
const layoutNodeId = (item: CanvasNodeLayoutV1 | string) => (typeof item === "string" ? item : item.nodeId);

interface CanvasNavigationContext {
  layouts(): readonly CanvasNodeLayoutV1[];
  selectedNodeIds(): ReadonlySet<string>;
  setSelection(ids: readonly string[]): void;
  setViewport(viewport: CanvasViewport): void;
  size(): { width: number; height: number };
  viewport(): CanvasViewport;
}

export function zoomCanvasViewport(context: CanvasNavigationContext, factor: number) {
  setCanvasZoom(context, context.viewport().zoom * factor);
}

export function setCanvasZoom(context: CanvasNavigationContext, zoom: number) {
  const size = context.size();
  context.setViewport(zoomViewportAt(context.viewport(), { x: size.width / 2, y: size.height / 2 }, zoom));
}

export function revealCanvasLayouts(
  context: CanvasNavigationContext,
  ids: readonly string[],
  mode: "ensure-visible" | "focus",
) {
  const layouts = intersectionBy(context.layouts(), ids, layoutNodeId);
  if (!layouts.length) return;
  context.setSelection(ids);
  const bounds = layoutBounds(layouts);
  if (mode === "focus") {
    fitCanvasLayouts(context, layouts);
  } else if (!isVisible(bounds, context.viewport(), context.size())) {
    context.setViewport(ensureBoundsVisible(bounds, context.viewport(), context.size()));
  }
}

export function revealCanvasTargets(
  context: CanvasNavigationContext & { setStructureSelection(ids: readonly string[]): void },
  ids: readonly string[],
  mode: "ensure-visible" | "focus",
  structureBounds: ReadonlyMap<string, { x: number; y: number; width: number; height: number }>,
) {
  const layouts = intersectionBy(context.layouts(), ids, layoutNodeId);
  if (layouts.length) return revealCanvasLayouts(context, ids, mode);
  const bounds = mergeBounds(compact(ids.map((id) => structureBounds.get(id))));
  if (!bounds) return;
  context.setStructureSelection(ids);
  if (mode === "focus") fitCanvasBounds(context, bounds);
  else if (!isVisible(bounds, context.viewport(), context.size()))
    context.setViewport(ensureBoundsVisible(bounds, context.viewport(), context.size()));
}

export function fitCanvasLayouts(context: CanvasNavigationContext, layouts: readonly CanvasNodeLayoutV1[]) {
  if (!layouts.length) return;
  fitCanvasBounds(context, layoutBounds(layouts));
}

export function fitCanvasBounds(
  context: Pick<CanvasNavigationContext, "setViewport" | "size" | "viewport">,
  bounds: { x: number; y: number; width: number; height: number },
) {
  const size = context.size();
  const padding = 80;
  const zoom = Math.min(
    (size.width - padding * 2) / bounds.width,
    (size.height - padding * 2) / bounds.height,
    2,
  );
  context.setViewport({
    x: (size.width - bounds.width * zoom) / 2 - bounds.x * zoom,
    y: (size.height - bounds.height * zoom) / 2 - bounds.y * zoom,
    zoom,
  });
}

function mergeBounds(bounds: readonly { x: number; y: number; width: number; height: number }[]) {
  if (!bounds.length) return undefined;
  const left = Math.min(...bounds.map(({ x }) => x));
  const top = Math.min(...bounds.map(({ y }) => y));
  const right = Math.max(...bounds.map(({ x, width }) => x + width));
  const bottom = Math.max(...bounds.map(({ y, height }) => y + height));
  return { x: left, y: top, width: Math.max(1, right - left), height: Math.max(1, bottom - top) };
}

export function selectedCanvasLayouts(context: CanvasNavigationContext) {
  const selected = context.selectedNodeIds();
  return context.layouts().filter(({ nodeId }) => selected.has(nodeId));
}

function layoutBounds(layouts: readonly CanvasNodeLayoutV1[]) {
  const left = Math.min(...layouts.map(({ frame }) => frame.x));
  const top = Math.min(...layouts.map(({ frame }) => frame.y));
  const right = Math.max(...layouts.map(({ frame }) => frame.x + frame.width));
  const bottom = Math.max(...layouts.map(({ frame }) => frame.y + frame.height));
  return { x: left, y: top, width: Math.max(1, right - left), height: Math.max(1, bottom - top) };
}

function isVisible(
  bounds: ReturnType<typeof layoutBounds>,
  viewport: CanvasViewport,
  size: { width: number; height: number },
) {
  const topLeft = screenPointFromCanvas(bounds, viewport);
  const bottomRight = screenPointFromCanvas(
    { x: bounds.x + bounds.width, y: bounds.y + bounds.height },
    viewport,
  );
  return topLeft.x >= 0 && topLeft.y >= 0 && bottomRight.x <= size.width && bottomRight.y <= size.height;
}

function ensureBoundsVisible(
  bounds: ReturnType<typeof layoutBounds>,
  viewport: CanvasViewport,
  size: { width: number; height: number },
) {
  const padding = 32;
  const topLeft = screenPointFromCanvas(bounds, viewport);
  const bottomRight = screenPointFromCanvas(
    { x: bounds.x + bounds.width, y: bounds.y + bounds.height },
    viewport,
  );
  const availableWidth = Math.max(0, size.width - padding * 2);
  const availableHeight = Math.max(0, size.height - padding * 2);
  const centerX = (topLeft.x + bottomRight.x) / 2;
  const centerY = (topLeft.y + bottomRight.y) / 2;
  const x =
    bottomRight.x - topLeft.x > availableWidth
      ? viewport.x + size.width / 2 - centerX
      : viewport.x + Math.max(padding - topLeft.x, Math.min(0, size.width - padding - bottomRight.x));
  const y =
    bottomRight.y - topLeft.y > availableHeight
      ? viewport.y + size.height / 2 - centerY
      : viewport.y + Math.max(padding - topLeft.y, Math.min(0, size.height - padding - bottomRight.y));
  return { x, y, zoom: viewport.zoom };
}

import type { CanvasRect, CanvasViewport } from "../../contracts/canvas-editor-port";

interface CullableNode {
  layout: { frame: CanvasRect; nodeId: string };
}

interface CullableRelation {
  bounds: CanvasRect;
  key: string;
}

interface ViewportSize {
  height: number;
  width: number;
}

export function visibleSceneRelationIds(
  relations: readonly CullableRelation[],
  viewport: CanvasViewport,
  size: ViewportSize,
  overscanPixels = 240,
) {
  const visible = worldBounds(viewport, size, overscanPixels);
  return new Set(relations.filter(({ bounds }) => intersects(bounds, visible)).map(({ key }) => key));
}

export function visibleSceneNodeIds(
  nodes: readonly CullableNode[],
  viewport: CanvasViewport,
  size: ViewportSize,
  overscanPixels = 240,
) {
  const visible = worldBounds(viewport, size, overscanPixels);
  return new Set(
    nodes.filter(({ layout }) => intersects(layout.frame, visible)).map(({ layout }) => layout.nodeId),
  );
}

function worldBounds(viewport: CanvasViewport, size: ViewportSize, overscanPixels: number): CanvasRect {
  const overscan = overscanPixels / viewport.zoom;
  return {
    x: -viewport.x / viewport.zoom - overscan,
    y: -viewport.y / viewport.zoom - overscan,
    width: size.width / viewport.zoom + overscan * 2,
    height: size.height / viewport.zoom + overscan * 2,
  };
}

function intersects(left: CanvasRect, right: CanvasRect) {
  return (
    left.x <= right.x + right.width &&
    left.x + left.width >= right.x &&
    left.y <= right.y + right.height &&
    left.y + left.height >= right.y
  );
}

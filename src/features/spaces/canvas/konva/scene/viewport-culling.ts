import type { CanvasViewport } from "../../contract/canvas-editor";

interface CullableNode {
  nodeId: string;
  frame: { x: number; y: number; width: number; height: number };
}

interface CullableRelation {
  bounds: { x: number; y: number; width: number; height: number };
  key: string;
}

interface ViewportSize {
  width: number;
  height: number;
}

export function visibleSceneRelationIds(
  relations: readonly CullableRelation[],
  viewport: CanvasViewport,
  size: ViewportSize,
  overscanPixels = 240,
) {
  const overscan = overscanPixels / viewport.zoom;
  const bounds = {
    x: -viewport.x / viewport.zoom - overscan,
    y: -viewport.y / viewport.zoom - overscan,
    width: size.width / viewport.zoom + overscan * 2,
    height: size.height / viewport.zoom + overscan * 2,
  };
  return new Set(relations.filter((relation) => intersects(relation.bounds, bounds)).map(({ key }) => key));
}

export function visibleSceneNodeIds(
  nodes: readonly CullableNode[],
  viewport: CanvasViewport,
  size: ViewportSize,
  overscanPixels = 240,
) {
  const overscan = overscanPixels / viewport.zoom;
  const bounds = {
    x: -viewport.x / viewport.zoom - overscan,
    y: -viewport.y / viewport.zoom - overscan,
    width: size.width / viewport.zoom + overscan * 2,
    height: size.height / viewport.zoom + overscan * 2,
  };
  return new Set(nodes.filter(({ frame }) => intersects(frame, bounds)).map(({ nodeId }) => nodeId));
}

function intersects(
  left: { x: number; y: number; width: number; height: number },
  right: { x: number; y: number; width: number; height: number },
) {
  return (
    left.x <= right.x + right.width &&
    left.x + left.width >= right.x &&
    left.y <= right.y + right.height &&
    left.y + left.height >= right.y
  );
}

import RBush, { type BBox } from "rbush";

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

interface IndexedSceneBounds extends BBox {
  id: string;
}

export function createSceneVisibilityIndex(
  nodes: readonly CullableNode[],
  relations: readonly CullableRelation[],
) {
  return {
    nodes: new RBush<IndexedSceneBounds>().load(
      nodes.map(({ layout }) => ({ ...bounds(layout.frame), id: layout.nodeId })),
    ),
    relations: new RBush<IndexedSceneBounds>().load(
      relations.map(({ bounds: frame, key }) => ({ ...bounds(frame), id: key })),
    ),
  };
}

export function visibleSceneIds(
  index: ReturnType<typeof createSceneVisibilityIndex>,
  viewport: CanvasViewport,
  size: ViewportSize,
  overscanPixels = 240,
) {
  const overscan = overscanPixels / viewport.zoom;
  const visible = bounds({
    x: -viewport.x / viewport.zoom - overscan,
    y: -viewport.y / viewport.zoom - overscan,
    width: size.width / viewport.zoom + overscan * 2,
    height: size.height / viewport.zoom + overscan * 2,
  });
  return {
    nodes: new Set(index.nodes.search(visible).map(({ id }) => id)),
    relations: new Set(index.relations.search(visible).map(({ id }) => id)),
  };
}

function bounds(frame: CanvasRect): BBox {
  return { minX: frame.x, minY: frame.y, maxX: frame.x + frame.width, maxY: frame.y + frame.height };
}

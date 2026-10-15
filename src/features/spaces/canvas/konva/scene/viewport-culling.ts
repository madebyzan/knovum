import RBush, { type BBox } from "rbush";

import { canvasFrameBounds } from "../../canvas-snap-geometry";
import type { CanvasRect, CanvasViewport } from "../../contracts/canvas-editor-port";
import type { CanvasNodeFrame } from "../../schema/document";

interface CullableNode {
  layout: { frame: CanvasNodeFrame; nodeId: string };
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

/** Screen-space band kept around the viewport, per side, before the index is queried. */
export interface SceneOverscan {
  bottom: number;
  left: number;
  right: number;
  top: number;
}

const BASE_OVERSCAN_PX = 240;
/** Samples of travel the leading sides run ahead of the camera. */
const LEAD_SAMPLES = 2;
const MAX_LEAD_PX = 960;

function lead(screenDelta: number) {
  return Math.min(Math.max(screenDelta, 0) * LEAD_SAMPLES, MAX_LEAD_PX);
}

/**
 * Overscan for a camera that moved `travel` screen pixels since the previous sample.
 *
 * A symmetric band spends three quarters of its budget behind and beside the camera. Growing the sides the
 * camera travels towards instead gives content a runway: it enters the visible set before it reaches the
 * viewport, which is what lets node content mount within a frame budget rather than on demand. The stage
 * moves opposite to the camera, so a rising stage position travels towards smaller world coordinates.
 */
export function sceneOverscan(travel: { x: number; y: number }): SceneOverscan {
  return {
    bottom: BASE_OVERSCAN_PX + lead(-travel.y),
    left: BASE_OVERSCAN_PX + lead(travel.x),
    right: BASE_OVERSCAN_PX + lead(-travel.x),
    top: BASE_OVERSCAN_PX + lead(travel.y),
  };
}

export function createSceneVisibilityIndex(
  nodes: readonly CullableNode[],
  relations: readonly CullableRelation[],
  sections: readonly CullableRelation[],
) {
  return {
    nodes: new RBush<IndexedSceneBounds>().load(
      nodes.map(({ layout }) => {
        const frame = canvasFrameBounds(layout.frame);
        return {
          minX: frame.left,
          minY: frame.top,
          maxX: frame.right,
          maxY: frame.bottom,
          id: layout.nodeId,
        };
      }),
    ),
    sections: new RBush<IndexedSceneBounds>().load(
      sections.map(({ bounds: frame, key }) => ({ ...bounds(frame), id: key })),
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
  overscan: SceneOverscan = sceneOverscan({ x: 0, y: 0 }),
) {
  const visible = bounds({
    x: (-viewport.x - overscan.left) / viewport.zoom,
    y: (-viewport.y - overscan.top) / viewport.zoom,
    width: (size.width + overscan.left + overscan.right) / viewport.zoom,
    height: (size.height + overscan.top + overscan.bottom) / viewport.zoom,
  });
  return {
    nodes: new Set(index.nodes.search(visible).map(({ id }) => id)),
    sections: new Set(index.sections.search(visible).map(({ id }) => id)),
    relations: new Set(index.relations.search(visible).map(({ id }) => id)),
  };
}

function bounds(frame: CanvasRect): BBox {
  return { minX: frame.x, minY: frame.y, maxX: frame.x + frame.width, maxY: frame.y + frame.height };
}

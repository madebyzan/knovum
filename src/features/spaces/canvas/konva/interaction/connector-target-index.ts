import { maxBy } from "es-toolkit";
import RBush from "rbush";

import { canvasFrameBounds } from "../../canvas-snap-geometry";
import type { CanvasPoint } from "../../contracts/canvas-editor-port";
import type { CanvasNodeLayoutV1 } from "../../schema/document";
import { frameContainsPoint } from "../scene/connector-geometry";

interface IndexedConnectorTarget {
  frame: CanvasNodeLayoutV1["frame"];
  maxX: number;
  maxY: number;
  minX: number;
  minY: number;
  nodeId: string;
  zIndex: number;
}

export interface ConnectorTargetIndex {
  targetByNodeId: ReadonlyMap<string, IndexedConnectorTarget>;
  tree: RBush<IndexedConnectorTarget>;
}

export function createConnectorTargetIndex(layouts: readonly CanvasNodeLayoutV1[]): ConnectorTargetIndex {
  const targets = layouts.map(({ frame, nodeId, zIndex }) => {
    const bounds = canvasFrameBounds(frame);
    return {
      frame,
      nodeId,
      zIndex,
      minX: bounds.left,
      minY: bounds.top,
      maxX: bounds.right,
      maxY: bounds.bottom,
    } satisfies IndexedConnectorTarget;
  });
  return {
    targetByNodeId: new Map(targets.map((target) => [target.nodeId, target])),
    tree: new RBush<IndexedConnectorTarget>().load(targets),
  };
}

/** The tree narrows by axis-aligned bounds, so a rotated frame still needs an exact hit test. */
export function topmostTargetAt(index: ConnectorTargetIndex, point: CanvasPoint, excludeNodeId: string) {
  const covering = index.tree
    .search({ minX: point.x, minY: point.y, maxX: point.x, maxY: point.y })
    .filter(({ nodeId, frame }) => nodeId !== excludeNodeId && frameContainsPoint(frame, point));
  return maxBy(covering, ({ zIndex }) => zIndex);
}

/** Side anchors always fall inside the frame bounds, so padding the query by the radius cannot miss one. */
export function targetsWithin(
  index: ConnectorTargetIndex,
  point: CanvasPoint,
  radius: number,
  excludeNodeId: string,
) {
  return index.tree
    .search({
      minX: point.x - radius,
      minY: point.y - radius,
      maxX: point.x + radius,
      maxY: point.y + radius,
    })
    .filter(({ nodeId }) => nodeId !== excludeNodeId);
}

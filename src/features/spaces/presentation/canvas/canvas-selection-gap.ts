import { canvasChromeMetrics } from "../../canvas/canvas-chrome-metrics";
import { isCanvasNodeConnectable } from "../../canvas/canvas-node-interaction";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";

const { connector, group } = canvasChromeMetrics(1);
const connectorGap = Math.ceil(connector.offset + connector.radius + connector.hitExtension / 2 + 4);

const groupGap = Math.ceil(group.rotationOffset + group.handleRadius + group.hitStrokeWidth / 2 + 4);

export function canvasSelectionGap(node: SpaceCanvasNode | undefined, hasGroup: boolean): number {
  const contentGap = node && isCanvasNodeConnectable(node) ? connectorGap : 12;
  return hasGroup ? Math.max(contentGap, groupGap) : contentGap;
}

import type { SpaceCanvasNode } from "../../../model/nodes/canvas-node";
import { canvasChromeMetrics } from "../../canvas-chrome-metrics";
import { isCanvasNodeConnectable } from "../../canvas-node-interaction";

const { connector, group } = canvasChromeMetrics(1);
const connectorGap = Math.ceil(connector.offset + connector.radius + connector.hitExtension / 2 + 4);

const groupGap = Math.ceil(group.rotationOffset + group.handleRadius + group.hitStrokeWidth / 2 + 4);

export function canvasSelectionGap(node: SpaceCanvasNode | undefined, hasGroup: boolean): number {
  const contentGap = node && isCanvasNodeConnectable(node) ? connectorGap : 12;
  return hasGroup ? Math.max(contentGap, groupGap) : contentGap;
}

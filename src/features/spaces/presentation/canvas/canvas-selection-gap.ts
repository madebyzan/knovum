import { canvasChromeMetrics } from "../../canvas/canvas-chrome-metrics";
import { isCanvasNodeConnectable } from "../../canvas/canvas-node-interaction";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";

const connector = canvasChromeMetrics(1).connector;
const connectorGap = Math.ceil(connector.offset + connector.radius + connector.hitExtension / 2 + 4);

export function canvasSelectionGap(node?: SpaceCanvasNode): number {
  return node && isCanvasNodeConnectable(node) ? connectorGap : 12;
}

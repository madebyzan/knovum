import type { CanvasNodePlacement } from "../canvas/contracts/canvas-editor-port";
import { spaceNodeDimensions } from "../canvas/node-size";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";

export function derivedNodePosition(
  nodes: readonly SpaceCanvasNode[],
  placements: readonly CanvasNodePlacement[],
) {
  const positionByNodeId = new Map(placements.map(({ id, x, y }) => [id, { x, y }]));
  const positioned = nodes.map((node) => ({ node, position: requiredPosition(positionByNodeId, node.id) }));
  const right = Math.max(
    ...positioned.map(({ node, position }) => position.x + spaceNodeDimensions(node.data).width),
  );
  const top = Math.min(...positioned.map(({ position }) => position.y));
  return { x: right + 48, y: top };
}

function requiredPosition(positionByNodeId: ReadonlyMap<string, { x: number; y: number }>, nodeId: string) {
  const position = positionByNodeId.get(nodeId);
  if (!position) throw new Error(`Current canvas placement is unavailable for node ${nodeId}`);
  return position;
}

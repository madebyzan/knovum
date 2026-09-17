import type { CanvasNodePlacement } from "../canvas/contract/canvas-editor";
import { spaceNodeDimensions } from "../canvas/node-size";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";

export function derivedNodePosition(
  nodes: readonly SpaceCanvasNode[],
  placements: readonly CanvasNodePlacement[],
) {
  const positions = new Map(placements.map(({ id, x, y }) => [id, { x, y }]));
  const positioned = nodes.map((node) => ({ node, position: requiredPosition(positions, node.id) }));
  const right = Math.max(
    ...positioned.map(({ node, position }) => position.x + spaceNodeDimensions(node.data).width),
  );
  const top = Math.min(...positioned.map(({ position }) => position.y));
  return { x: right + 48, y: top };
}

export function requiredNodePosition(placements: readonly CanvasNodePlacement[], nodeId: string) {
  return requiredPosition(new Map(placements.map(({ id, x, y }) => [id, { x, y }])), nodeId);
}

function requiredPosition(positions: ReadonlyMap<string, { x: number; y: number }>, nodeId: string) {
  const position = positions.get(nodeId);
  if (!position) throw new Error(`Current canvas placement is unavailable for node ${nodeId}`);
  return position;
}

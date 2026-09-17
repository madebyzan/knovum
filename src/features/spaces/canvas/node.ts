import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceNodePayload } from "../model/nodes/node-payload";
import { validateSpaceNodePayload } from "../model/nodes/schema/node-payload";

export function createSpaceNode(
  data: SpaceNodePayload,
  initialPosition: { x: number; y: number },
  id: string = crypto.randomUUID(),
): SpaceCanvasNode {
  return applySpaceNodeData({ id, type: data.kind, initialPosition, data }, data);
}

export function applySpaceNodeData(node: SpaceCanvasNode, data: SpaceNodePayload): SpaceCanvasNode {
  return {
    ...node,
    data,
    type: data.kind,
  };
}

export function snapshotSpaceNode(node: SpaceCanvasNode): SpaceCanvasNode {
  return createSpaceNode(validateSpaceNodePayload(node.data), { ...node.initialPosition }, node.id);
}

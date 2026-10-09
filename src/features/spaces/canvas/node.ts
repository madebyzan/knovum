import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceNodePayload } from "../model/nodes/node-payload";

export function createSpaceNode<T extends SpaceNodePayload>(
  data: T,
  initialPosition: { x: number; y: number },
  nodeId: string = crypto.randomUUID(),
): SpaceCanvasNode<T> {
  // The constructor keeps the redundant canvas discriminator in sync with the payload.
  return { id: nodeId, type: data.kind, initialPosition, data } as SpaceCanvasNode<T>;
}

export function applySpaceNodeContent(node: SpaceCanvasNode, data: SpaceNodePayload): SpaceCanvasNode {
  return {
    ...node,
    data,
    type: data.kind,
  } as SpaceCanvasNode;
}

import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceNodePayload } from "../model/nodes/node-payload";
import { validateSpaceNodePayload } from "../model/nodes/node-validation";

export function createSpaceNode<T extends SpaceNodePayload>(
  data: T,
  initialPosition: { x: number; y: number },
  id: string = crypto.randomUUID(),
): SpaceCanvasNode<T> {
  // The constructor keeps the redundant canvas discriminator in sync with the payload.
  return { id, type: data.kind, initialPosition, data } as SpaceCanvasNode<T>;
}

export function applySpaceNodeContent(node: SpaceCanvasNode, data: SpaceNodePayload): SpaceCanvasNode {
  return {
    ...node,
    data,
    type: data.kind,
  } as SpaceCanvasNode;
}

export function snapshotSpaceNode(node: SpaceCanvasNode): SpaceCanvasNode {
  return createSpaceNode(validateSpaceNodePayload(node.data), { ...node.initialPosition }, node.id);
}

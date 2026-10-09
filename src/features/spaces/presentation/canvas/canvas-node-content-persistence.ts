import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceNodePayload } from "../../model/nodes/node-payload";
import type { NodePayloadChange } from "../../model/nodes/payload-change";

interface CanvasNodeContentPersistence {
  nodes: readonly SpaceCanvasNode[];
  updateNodeContents(changes: readonly NodePayloadChange[]): Promise<void>;
}

export function createCanvasNodeContentPersistence(store: CanvasNodeContentPersistence) {
  return async (nodeId: string, after: SpaceNodePayload) => {
    const before = store.nodes.find((node) => node.id === nodeId)?.data;
    if (!before) throw new Error(`Cannot persist missing canvas node ${nodeId}`);
    await store.updateNodeContents([{ id: nodeId, before, after }]);
  };
}

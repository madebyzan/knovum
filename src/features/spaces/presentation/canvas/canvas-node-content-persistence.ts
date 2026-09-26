import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceNodePayload } from "../../model/nodes/node-payload";
import type { NodePayloadChange } from "../../session/actions/node-mutations";

interface CanvasNodeContentPersistence {
  nodes: readonly SpaceCanvasNode[];
  updateManyData(changes: readonly NodePayloadChange[]): Promise<void>;
}

export function createCanvasNodeContentPersistence(store: CanvasNodeContentPersistence) {
  return async (id: string, after: SpaceNodePayload) => {
    const before = store.nodes.find((node) => node.id === id)?.data;
    if (!before) throw new Error(`Cannot persist missing canvas node ${id}`);
    await store.updateManyData([{ id, before, after }]);
  };
}

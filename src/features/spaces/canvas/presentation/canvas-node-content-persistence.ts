import type { SpaceNodePayload } from "../../model/nodes/node-payload";
import { useSpaceStore } from "../../session/space-store";

export function createCanvasNodeContentPersistence() {
  const store = useSpaceStore();

  return async (nodeId: string, after: SpaceNodePayload) => {
    const before = store.nodes.find((node) => node.id === nodeId)?.data;
    if (!before) throw new Error(`Cannot persist missing canvas node ${nodeId}`);
    await store.updateNodeContents([{ id: nodeId, before, after }]);
  };
}

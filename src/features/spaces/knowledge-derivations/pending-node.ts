import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { useSpaceStore } from "../session/space-store";

type Store = ReturnType<typeof useSpaceStore>;

export async function markProcessingNodeFailed(store: Store, spaceId: string, node: SpaceCanvasNode) {
  if (!("processingState" in node.data)) return;
  await store.saveNodeDataForSpace(spaceId, node.id, node.data, { ...node.data, processingState: "failed" });
}

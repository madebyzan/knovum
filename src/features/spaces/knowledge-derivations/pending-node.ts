import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { useSpaceStore } from "../session/space-store";

type Store = Pick<ReturnType<typeof useSpaceStore>, "saveNodeContentForSpace">;

export async function markProcessingNodeFailed(store: Store, spaceId: string, node: SpaceCanvasNode) {
  if (!("processingState" in node.data)) return;
  await store.saveNodeContentForSpace(spaceId, {
    id: node.id,
    expectedData: node.data,
    data: { ...node.data, processingState: "failed" },
  });
}

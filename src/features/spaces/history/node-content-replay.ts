import type { Ref } from "vue";

import { applySpaceNodeContent } from "../canvas/node";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceNodePayload } from "../model/nodes/node-payload";
import { updateNodePayloadMany } from "../nodes/node-operations";

interface MutationProjection {
  nodes: Ref<SpaceCanvasNode[]>;
  activeSpaceId: Ref<string | undefined>;
}

interface ContentReplayInput extends MutationProjection {
  spaceId: string;
  changes: readonly { id: string; expectedData: SpaceNodePayload; data: SpaceNodePayload }[];
}

export async function replayNodeContent(input: ContentReplayInput) {
  const { spaceId, changes, nodes, activeSpaceId } = input;
  const committed = await updateNodePayloadMany(spaceId, changes);
  if (activeSpaceId.value !== spaceId) return committed;
  const committedContent = new Map(committed.map((change) => [change.id, change.data]));
  nodes.value = nodes.value.map((node: SpaceCanvasNode) => {
    const data = committedContent.get(node.id);
    return data ? applySpaceNodeContent(node, data) : node;
  });
  return committed;
}

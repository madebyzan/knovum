import type { Ref } from "vue";

import { applySpaceNodeContent } from "../canvas/node";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceNodePayload } from "../model/nodes/node-payload";
import type { SpaceEditSnapshot } from "../model/structure/schema/edit-snapshot";
import { updateNodePayloadMany } from "../nodes/node-operations";

interface MutationProjection {
  nodes: Ref<SpaceCanvasNode[]>;
  editSnapshot: Ref<SpaceEditSnapshot>;
  activeSpaceId: Ref<string | undefined>;
}

interface ContentReplayInput extends MutationProjection {
  spaceId: string;
  changes: readonly { id: string; expectedData: SpaceNodePayload; data: SpaceNodePayload }[];
}

export async function replayNodeContent(input: ContentReplayInput) {
  const { spaceId, changes, nodes, editSnapshot, activeSpaceId } = input;
  const committed = await updateNodePayloadMany(spaceId, changes);
  if (activeSpaceId.value !== spaceId) return committed.nodes;
  const payloadByNodeId = new Map(committed.nodes.map((change) => [change.id, change.data]));
  nodes.value = nodes.value.map((node: SpaceCanvasNode) => {
    const data = payloadByNodeId.get(node.id);
    return data ? applySpaceNodeContent(node, data) : node;
  });
  if (committed.editSnapshot && committed.editSnapshot.editRevision > editSnapshot.value.editRevision)
    editSnapshot.value = committed.editSnapshot;
  return committed.nodes;
}

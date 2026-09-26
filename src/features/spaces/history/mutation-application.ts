import type { Ref } from "vue";

import { applySpaceNodeData } from "../canvas/node";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceNodePayload } from "../model/nodes/node-payload";
import { updateNodePayloadMany } from "../nodes/node-operations";

interface MutationProjection {
  nodes: Ref<SpaceCanvasNode[]>;
  activeSpaceId: Ref<string | undefined>;
}

interface ApplyDataInput extends MutationProjection {
  spaceId: string;
  changes: readonly { id: string; expectedData: SpaceNodePayload; data: SpaceNodePayload }[];
}

export async function applyData(input: ApplyDataInput) {
  const { spaceId, changes, nodes, activeSpaceId } = input;
  await updateNodePayloadMany(spaceId, changes);
  if (activeSpaceId.value !== spaceId) return;
  const nextData = new Map(changes.map((change) => [change.id, change.data]));
  nodes.value = nodes.value.map((node: SpaceCanvasNode) => {
    const data = nextData.get(node.id);
    return data ? applySpaceNodeData(node, data) : node;
  });
}

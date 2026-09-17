import type { Ref } from "vue";

import { applySpaceNodeData, snapshotSpaceNode } from "../canvas/node";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceNodePayload } from "../model/nodes/node-payload";
import { setNodesDeleted, updateNodePayloadMany } from "../nodes/node-repository";
import type { SpaceHistoryEntry } from "./stack";

interface MutationProjection {
  nodes: Ref<SpaceCanvasNode[]>;
  activeSpaceId: Ref<string | undefined>;
}

interface ApplyDataInput extends MutationProjection {
  spaceId: string;
  changes: readonly { id: string; data: SpaceNodePayload }[];
}

export function createAddHistory(
  spaceId: string,
  items: readonly SpaceCanvasNode[],
  nodes: Ref<SpaceCanvasNode[]>,
  activeSpaceId: Ref<string | undefined>,
): SpaceHistoryEntry {
  const snapshots = items.map(snapshotSpaceNode);
  return {
    undo: async () => {
      await setNodesDeleted(
        spaceId,
        snapshots.map((node) => node.id),
        true,
      );
      if (activeSpaceId.value === spaceId)
        nodes.value = nodes.value.filter(
          (node: SpaceCanvasNode) => !snapshots.some((snapshot) => snapshot.id === node.id),
        );
    },
    redo: async () => {
      await setNodesDeleted(
        spaceId,
        snapshots.map((node) => node.id),
        false,
      );
      if (activeSpaceId.value === spaceId)
        nodes.value = [...nodes.value, ...snapshots.map(snapshotSpaceNode)];
    },
  };
}

export function createDeleteHistory(
  spaceId: string,
  items: readonly SpaceCanvasNode[],
  nodes: Ref<SpaceCanvasNode[]>,
  activeSpaceId: Ref<string | undefined>,
): SpaceHistoryEntry {
  const snapshots = items.map(snapshotSpaceNode);
  return {
    undo: async () => {
      await setNodesDeleted(
        spaceId,
        snapshots.map((node) => node.id),
        false,
      );
      if (activeSpaceId.value === spaceId)
        nodes.value = [...nodes.value, ...snapshots.map(snapshotSpaceNode)];
    },
    redo: async () => {
      await setNodesDeleted(
        spaceId,
        snapshots.map((node) => node.id),
        true,
      );
      if (activeSpaceId.value === spaceId)
        nodes.value = nodes.value.filter(
          (node: SpaceCanvasNode) => !snapshots.some((snapshot) => snapshot.id === node.id),
        );
    },
  };
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

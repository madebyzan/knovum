import type { Ref } from "vue";

import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { SpaceRelation } from "../model/relations/relation";
import { moveNodesBetweenSpaces } from "./node-repository";

interface NodeLifecycleContext {
  activeSpaceId: Ref<string | undefined>;
  nodes: Ref<SpaceCanvasNode[]>;
  relations: Ref<SpaceRelation[]>;
}

export function createSpaceNodeLifecycleActions(context: NodeLifecycleContext) {
  async function moveNodes(ids: readonly string[], targetSpaceId: string) {
    const sourceSpaceId = context.activeSpaceId.value;
    if (!sourceSpaceId || sourceSpaceId === targetSpaceId) return [];
    const moved = await moveNodesBetweenSpaces(sourceSpaceId, targetSpaceId, ids);
    if (context.activeSpaceId.value === sourceSpaceId && moved.length) {
      const movedSet = new Set(moved);
      context.nodes.value = context.nodes.value.filter((node) => !movedSet.has(node.id));
      context.relations.value = context.relations.value.filter(
        (relation) => !movedSet.has(relation.sourceId) && !movedSet.has(relation.targetId),
      );
    }
    return moved;
  }

  return { moveNodes };
}

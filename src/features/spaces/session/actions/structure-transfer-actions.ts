import type { Ref } from "vue";

import type { SpaceEditSnapshot } from "../../canvas/contract/space-edit-snapshot";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceRelation } from "../../model/relations/relation";
import { spaceEditOperations } from "../../structure/space-edit-operations";
import { moveStructureSelectionBetweenSpaces } from "../../structure/structure-transfer-operations";
import { loadSpaceSessionProjection } from "../space-projection";

interface StructureTransferContext {
  activeSpaceId: Ref<string | undefined>;
  editSnapshot: Ref<SpaceEditSnapshot>;
  nodes: Ref<SpaceCanvasNode[]>;
  relations: Ref<SpaceRelation[]>;
  recordCommittedSpaceEdit(): void;
}

export function createStructureTransferActions(context: StructureTransferContext) {
  async function moveStructureNodes(rootStructureNodeIds: readonly string[], targetSpaceId: string) {
    const sourceSpaceId = context.activeSpaceId.value;
    if (!sourceSpaceId || sourceSpaceId === targetSpaceId || !rootStructureNodeIds.length) return [];
    const sourceSnapshot = context.editSnapshot.value;
    const targetSnapshot = await spaceEditOperations.load(targetSpaceId);
    const targetOrder = targetSnapshot.structure.filter(({ parentId }) => parentId === null).length;
    const result = await moveStructureSelectionBetweenSpaces({
      sourceSpaceId,
      mutationId: crypto.randomUUID(),
      targetSpaceId,
      rootStructureNodeIds: [...rootStructureNodeIds],
      targetParentId: null,
      targetOrder,
      expectedSourceEditRevision: sourceSnapshot.editRevision,
      expectedTargetEditRevision: targetSnapshot.editRevision,
    });
    if (context.activeSpaceId.value === sourceSpaceId && result.movedNodeIds.length) {
      const projection = await loadSpaceSessionProjection(sourceSpaceId);
      if (context.activeSpaceId.value === sourceSpaceId) {
        context.nodes.value = projection.nodes;
        context.relations.value = projection.relations;
        context.editSnapshot.value = projection.editSnapshot;
        context.recordCommittedSpaceEdit();
      }
    }
    return result.movedStructureNodeIds;
  }

  return { moveStructureNodes };
}

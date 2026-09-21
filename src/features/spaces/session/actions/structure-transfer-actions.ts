import type { Ref } from "vue";

import type { SpaceEditSnapshot } from "../../canvas/contract/space-edit-snapshot";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceRelation } from "../../model/relations/relation";
import { spaceEditRepository } from "../../structure/space-edit-repository";
import { moveStructureSelectionBetweenSpaces } from "../../structure/structure-transfer-repository";

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
    const targetSnapshot = await spaceEditRepository.load(targetSpaceId);
    const targetOrder = targetSnapshot.structure.filter(({ parentId }) => parentId === null).length;
    const result = await moveStructureSelectionBetweenSpaces({
      sourceSpaceId,
      mutationId: crypto.randomUUID(),
      targetSpaceId,
      rootStructureNodeIds,
      targetParentId: null,
      targetOrder,
      expectedSourceEditRevision: sourceSnapshot.editRevision,
      expectedTargetEditRevision: targetSnapshot.editRevision,
    });
    if (context.activeSpaceId.value === sourceSpaceId && result.movedNodeIds.length) {
      const refreshedSnapshot = await spaceEditRepository.load(sourceSpaceId);
      const movedNodeIds = new Set(result.movedNodeIds);
      context.editSnapshot.value = refreshedSnapshot;
      context.nodes.value = context.nodes.value.filter((node) => !movedNodeIds.has(node.id));
      context.relations.value = context.relations.value.filter(
        (relation) => !movedNodeIds.has(relation.sourceId) && !movedNodeIds.has(relation.targetId),
      );
      context.recordCommittedSpaceEdit();
    }
    return result.movedStructureNodeIds;
  }

  return { moveStructureNodes };
}

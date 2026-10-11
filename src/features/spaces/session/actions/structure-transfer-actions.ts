import type { Ref } from "vue";

import { beginSpaceMutation } from "../../history/mutation-barrier";
import type { SpaceHistoryState } from "../../history/stack";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceRelation } from "../../model/relations/relation";
import type { SpaceEditSnapshot } from "../../model/structure/schema/edit-snapshot";
import { spaceEditOperations } from "../../structure/space-edit-operations";
import { moveStructureSelectionBetweenSpaces } from "../../structure/structure-transfer-operations";
import { assertEditorResourcesRemovable, editorResources } from "../../workspace/editor-resource-protection";
import type { SpaceEditorTab } from "../../workspace/work-items";
import { loadSpaceSessionProjection } from "../space-projection";

interface StructureTransferContext {
  activeSpaceId: Ref<string | undefined>;
  editSnapshot: Ref<SpaceEditSnapshot>;
  nodes: Ref<SpaceCanvasNode[]>;
  relations: Ref<SpaceRelation[]>;
  recordCommittedSpaceEdit(): void;
  editorTabs: Ref<readonly SpaceEditorTab[]>;
  history: SpaceHistoryState;
}

export function createStructureTransferActions(context: StructureTransferContext) {
  async function moveStructureNodes(rootStructureNodeIds: readonly string[], targetSpaceId: string) {
    const sourceSpaceId = context.activeSpaceId.value;
    if (!sourceSpaceId || sourceSpaceId === targetSpaceId || !rootStructureNodeIds.length) return [];
    assertEditorResourcesRemovable(
      context.editorTabs.value,
      sourceSpaceId,
      context.editSnapshot.value,
      rootStructureNodeIds,
    );
    const releaseMutation = beginSpaceMutation(context.history);
    try {
      if (context.activeSpaceId.value !== sourceSpaceId) throw new Error("Transfer source Space changed");
      const sourceSnapshot = context.editSnapshot.value;
      const targetSnapshot = await spaceEditOperations.load(targetSpaceId);
      const targetOrder = targetSnapshot.structure.filter(({ parentId }) => parentId === null).length;
      const transferReceipt = await moveStructureSelectionBetweenSpaces(
        {
          sourceSpaceId,
          mutationId: crypto.randomUUID(),
          targetSpaceId,
          rootStructureNodeIds: [...rootStructureNodeIds],
          targetParentId: null,
          targetOrder,
          expectedSourceEditRevision: sourceSnapshot.editRevision,
          expectedTargetEditRevision: targetSnapshot.editRevision,
        },
        editorResources(context.editorTabs.value),
      );
      if (context.activeSpaceId.value === sourceSpaceId && transferReceipt.movedNodeIds.length) {
        const projection = await loadSpaceSessionProjection(sourceSpaceId);
        if (context.activeSpaceId.value === sourceSpaceId) {
          context.nodes.value = projection.nodes;
          context.relations.value = projection.relations;
          context.editSnapshot.value = projection.editSnapshot;
          context.recordCommittedSpaceEdit();
        }
      }
      return transferReceipt.movedStructureNodeIds;
    } finally {
      releaseMutation();
    }
  }

  return { moveStructureNodes };
}

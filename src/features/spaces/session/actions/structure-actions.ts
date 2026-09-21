import type { Ref } from "vue";

import type { SpaceEditSnapshot } from "../../canvas/contract/space-edit-snapshot";
import { beginSpaceMutation } from "../../history/mutation-barrier";
import { recordSpaceHistory, type SpaceHistoryState } from "../../history/stack";
import { listRelations } from "../../knowledge/relations/relation-repository";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceRelation } from "../../model/relations/relation";
import { listNodes } from "../../nodes/node-repository";
import { spaceEditCommandRecordsHistory, type SpaceEditCommand } from "../../organization/space-edit-command";
import { spaceEditRepository } from "../../structure/space-edit-repository";

export function createStructureActions(context: {
  activeSpaceId: Ref<string | undefined>;
  editSnapshot: Ref<SpaceEditSnapshot>;
  nodes: Ref<SpaceCanvasNode[]>;
  relations: Ref<SpaceRelation[]>;
  history: SpaceHistoryState;
  ensureActiveSpacePersisted: () => Promise<unknown>;
}) {
  async function executeSpaceEdit(command: SpaceEditCommand) {
    await context.ensureActiveSpacePersisted();
    const spaceId = context.activeSpaceId.value;
    if (!spaceId) throw new Error("A Space must be active");
    const releaseMutation = beginSpaceMutation(context.history);
    try {
      const receipt = await spaceEditRepository.execute({
        spaceId,
        mutationId: crypto.randomUUID(),
        expectedEditRevision: context.editSnapshot.value.editRevision,
        command,
      });
      await reloadProjection(spaceId);
      if (spaceEditCommandRecordsHistory(command)) recordNativeHistoryEntry(spaceId);
      return receipt;
    } finally {
      releaseMutation();
    }
  }

  function recordCommittedSpaceEdit() {
    const spaceId = context.activeSpaceId.value;
    if (!spaceId) throw new Error("A Space must be active");
    recordNativeHistoryEntry(spaceId);
  }

  function recordNativeHistoryEntry(spaceId: string) {
    recordSpaceHistory(context.history, {
      undo: () => transition(spaceId, "undo"),
      redo: () => transition(spaceId, "redo"),
    });
  }

  async function transition(spaceId: string, direction: "undo" | "redo") {
    if (context.activeSpaceId.value !== spaceId)
      throw new Error("Open the Space where this edit was made before changing its history");
    await spaceEditRepository[direction](spaceId, context.editSnapshot.value.editRevision);
    await reloadProjection(spaceId);
  }

  async function reloadProjection(spaceId: string) {
    const [snapshot, nodes, relations] = await Promise.all([
      spaceEditRepository.load(spaceId),
      listNodes(spaceId),
      listRelations(spaceId),
    ]);
    if (context.activeSpaceId.value !== spaceId) return;
    context.editSnapshot.value = snapshot;
    context.nodes.value = nodes;
    context.relations.value = relations;
  }

  return { executeSpaceEdit, recordCommittedSpaceEdit };
}

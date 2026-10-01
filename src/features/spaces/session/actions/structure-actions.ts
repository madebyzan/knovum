import type { Ref } from "vue";

import { decodeCommandError } from "../../../../foundation/ipc/command-error";
import type { SpaceEditCommand } from "../../contracts/generated/SpaceEditCommand";
import { beginSpaceMutation } from "../../history/mutation-barrier";
import { recordSpaceHistory, type SpaceHistoryState } from "../../history/stack";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceRelation } from "../../model/relations/relation";
import type { SpaceEditSnapshot } from "../../model/structure/schema/edit-snapshot";
import { spaceEditCommandRecordsHistory } from "../../structure/space-edit-command";
import { spaceEditOperations } from "../../structure/space-edit-operations";
import { loadSpaceSessionProjection } from "../space-projection";

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
      const receipt = await spaceEditOperations.execute({
        spaceId,
        mutationId: crypto.randomUUID(),
        expectedEditRevision: context.editSnapshot.value.editRevision,
        command,
      });
      await reloadProjection(spaceId);
      if (spaceEditCommandRecordsHistory(command)) recordRustEdit(spaceId);
      return receipt;
    } catch (error) {
      if (decodeCommandError(error)?.code === "space_edit_revision_conflict") await reloadProjection(spaceId);
      throw error;
    } finally {
      releaseMutation();
    }
  }

  function recordCommittedSpaceEdit() {
    const spaceId = context.activeSpaceId.value;
    if (!spaceId) throw new Error("A Space must be active");
    recordRustEdit(spaceId);
  }

  function recordRustEdit(spaceId: string) {
    recordSpaceHistory(context.history, {
      undo: () => transition(spaceId, "undo"),
      redo: () => transition(spaceId, "redo"),
    });
  }

  async function transition(spaceId: string, direction: "undo" | "redo") {
    if (context.activeSpaceId.value !== spaceId)
      throw new Error("Open the Space where this edit was made before changing its history");
    try {
      await spaceEditOperations[direction](spaceId, context.editSnapshot.value.editRevision);
      await reloadProjection(spaceId);
    } catch (error) {
      if (decodeCommandError(error)?.code === "space_edit_revision_conflict") await reloadProjection(spaceId);
      throw error;
    }
  }

  async function reloadProjection(spaceId: string) {
    const { editSnapshot, nodes, relations } = await loadSpaceSessionProjection(spaceId);
    if (context.activeSpaceId.value !== spaceId) return;
    context.nodes.value = nodes;
    context.relations.value = relations;
    context.editSnapshot.value = editSnapshot;
  }

  return { executeSpaceEdit, recordCommittedSpaceEdit };
}

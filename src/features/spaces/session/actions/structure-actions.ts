import { isEqual } from "es-toolkit";
import type { Ref } from "vue";

import { decodeAppError } from "../../../../foundation/errors/app-error";
import type { SpaceEditCommand } from "../../contracts/generated/SpaceEditCommand";
import { beginSpaceMutation } from "../../history/mutation-barrier";
import { recordSpaceHistory, type SpaceHistoryState } from "../../history/stack";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceRelation } from "../../model/relations/relation";
import type { SpaceEditSnapshot } from "../../model/structure/schema/edit-snapshot";
import {
  spaceEditCommandProjection,
  spaceEditCommandRecordsHistory,
  type SpaceEditProjection,
} from "../../structure/space-edit-command";
import { spaceEditOperations } from "../../structure/space-edit-operations";
import { assertEditorResourcesRemovable, editorResources } from "../../workspace/editor-resource-protection";
import type { SpaceEditorTab } from "../../workspace/work-items";
import { loadSpaceSessionProjection } from "../space-projection";

interface StructureActionContext {
  activeSpaceId: Ref<string | undefined>;
  editSnapshot: Ref<SpaceEditSnapshot>;
  nodes: Ref<SpaceCanvasNode[]>;
  relations: Ref<SpaceRelation[]>;
  history: SpaceHistoryState;
  ensureActiveSpacePersisted: () => Promise<unknown>;
  editorTabs: Ref<readonly SpaceEditorTab[]>;
}

export function createStructureActions(context: StructureActionContext) {
  async function executeSpaceEdit(command: SpaceEditCommand) {
    await context.ensureActiveSpacePersisted();
    const spaceId = context.activeSpaceId.value;
    if (!spaceId) throw new Error("A Space must be active");
    if (command.kind === "structure.delete-subtrees")
      assertEditorResourcesRemovable(
        context.editorTabs.value,
        spaceId,
        context.editSnapshot.value,
        command.rootIds,
      );
    if (context.activeSpaceId.value !== spaceId) throw new Error("Structure edit Space changed");
    const releaseMutation = beginSpaceMutation(context.history);
    try {
      const receipt = await spaceEditOperations.execute(
        {
          spaceId,
          mutationId: crypto.randomUUID(),
          expectedEditRevision: context.editSnapshot.value.editRevision,
          command,
        },
        editorResources(context.editorTabs.value),
      );
      const projection = spaceEditCommandProjection(command);
      await reloadProjection(spaceId, projection);
      if (spaceEditCommandRecordsHistory(command)) recordRustEdit(spaceId, projection);
      return receipt;
    } catch (error) {
      if (decodeAppError(error)?.code === "space_edit_revision_conflict")
        await reloadProjection(spaceId, "session");
      throw error;
    } finally {
      releaseMutation();
    }
  }

  function recordCommittedSpaceEdit() {
    const spaceId = context.activeSpaceId.value;
    if (!spaceId) throw new Error("A Space must be active");
    recordRustEdit(spaceId, "session");
  }

  function recordRustEdit(spaceId: string, projection: SpaceEditProjection) {
    recordSpaceHistory(context.history, {
      undo: () => transition(spaceId, "undo", projection),
      redo: () => transition(spaceId, "redo", projection),
    });
  }

  async function transition(spaceId: string, direction: "undo" | "redo", projection: SpaceEditProjection) {
    if (context.activeSpaceId.value !== spaceId)
      throw new Error("Open the Space where this edit was made before changing its history");
    try {
      await spaceEditOperations[direction](
        spaceId,
        context.editSnapshot.value.editRevision,
        editorResources(context.editorTabs.value),
      );
      await reloadProjection(spaceId, projection);
    } catch (error) {
      if (decodeAppError(error)?.code === "space_edit_revision_conflict")
        await reloadProjection(spaceId, "session");
      throw error;
    }
  }

  async function reloadProjection(spaceId: string, projection: SpaceEditProjection) {
    const loadedProjection =
      projection === "session"
        ? await loadSpaceSessionProjection(spaceId)
        : { editSnapshot: await spaceEditOperations.load(spaceId) };
    if (context.activeSpaceId.value !== spaceId) return;
    if ("nodes" in loadedProjection) {
      context.nodes.value = loadedProjection.nodes;
      context.relations.value = loadedProjection.relations;
    }
    const current = context.editSnapshot.value;
    const next = loadedProjection.editSnapshot;
    context.editSnapshot.value = {
      ...next,
      structure: isEqual(current.structure, next.structure) ? current.structure : next.structure,
      relationStyles: isEqual(current.relationStyles, next.relationStyles)
        ? current.relationStyles
        : next.relationStyles,
    };
  }

  return { executeSpaceEdit, recordCommittedSpaceEdit };
}

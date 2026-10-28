import { clearSpaceHistory } from "../history/stack";
import type { SpaceSessionContent } from "../session/space-projection";
import type { SpaceSessionContext } from "./space-session-context";

/** The content of a Space that has none yet, such as an unsaved draft. */
export function emptySpaceContent(): SpaceSessionContent {
  return {
    nodes: [],
    scenes: [],
    relations: [],
    editSnapshot: { editRevision: 0, structure: [], layouts: [], relationStyles: [] },
  };
}

/** Retains the active Space's session, then makes `content` and the next Space's history current. */
export function switchSpaceSession(
  context: SpaceSessionContext,
  nextSpaceId: string,
  content: SpaceSessionContent,
) {
  const { retained } = context.runtime;
  const currentSpaceId = context.activeSpaceId.value;
  if (currentSpaceId) {
    retained.set(currentSpaceId, {
      content: {
        nodes: context.nodes.value,
        scenes: context.scenes.value,
        relations: context.relations.value,
        editSnapshot: context.editSnapshot.value,
      },
      undoStack: context.history.undoStack,
      redoStack: context.history.redoStack,
    });
  }
  const next = retained.get(nextSpaceId);
  retained.delete(nextSpaceId);
  if (next) {
    context.history.undoStack = next.undoStack;
    context.history.redoStack = next.redoStack;
  } else clearSpaceHistory(context.history);
  context.nodes.value = content.nodes;
  context.scenes.value = content.scenes;
  context.relations.value = content.relations;
  context.editSnapshot.value = content.editSnapshot;
  context.groupScopePath.value = [];
}

import { clearSpaceHistory } from "../history/stack";
import type { SpaceSessionContext } from "./space-session-context";

export function switchSpaceHistory(context: SpaceSessionContext, nextSpaceId: string) {
  const currentSpaceId = context.activeSpaceId.value;
  if (currentSpaceId) {
    context.runtime.historyBySpace.set(currentSpaceId, {
      undoStack: context.history.undoStack,
      redoStack: context.history.redoStack,
    });
  }
  const saved = context.runtime.historyBySpace.get(nextSpaceId);
  if (!saved) return clearSpaceHistory(context.history);
  context.history.undoStack = saved.undoStack;
  context.history.redoStack = saved.redoStack;
}

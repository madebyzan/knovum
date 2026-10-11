import { nextTick } from "vue";

import { runCanvasSessionTransition } from "../canvas/editor-session";
import { closeSpaceMutationAdmission, waitForSpaceMutations } from "../history/mutation-barrier";
import { flushSpaceSessionCommits } from "./session-participants";
import type { SpaceSessionContext } from "./space-session-context";
import { selectSpaceWorkItem } from "./work-items";

export function transitionActiveSpaceSession<T>(
  context: SpaceSessionContext,
  transition: () => Promise<T>,
  purpose: "switch" | "close" = "switch",
) {
  return runCanvasSessionTransition(async () => {
    let reopenMutations: (() => void) | undefined;
    try {
      await flushSpaceSessionCommits(
        async () => {
          reopenMutations = closeSpaceMutationAdmission(context.history);
          await waitForSpaceMutations(context.history);
        },
        purpose === "close"
          ? { purpose }
          : { purpose, spaceIds: context.activeSpaceId.value ? [context.activeSpaceId.value] : [] },
      );
      await nextTick();
      context.captureOverview();
      return await transition();
    } finally {
      reopenMutations?.();
    }
  });
}

export function commitActiveSpaceSession(context: SpaceSessionContext) {
  return transitionActiveSpaceSession(context, async () => {}, "close");
}

export function activateSpaceWorkItem(context: SpaceSessionContext, tabId?: string) {
  const spaceId = context.activeSpaceId.value;
  if (!spaceId) throw new Error("Space requires an active space");
  return transitionActiveSpaceSession(context, async () => {
    if (context.activeSpaceId.value !== spaceId) throw new Error("Work item Space changed");
    selectSpaceWorkItem(context, spaceId, tabId);
    await nextTick();
  });
}

import { nextTick } from "vue";

import { runCanvasSessionTransition } from "../canvas/editor-session";
import { closeSpaceMutationAdmission, waitForSpaceMutations } from "../history/mutation-barrier";
import { flushSpaceSessionCommits } from "./session-participants";
import type { SpaceSessionContext } from "./space-session-context";

export function transitionActiveSpaceSession<T>(context: SpaceSessionContext, transition: () => Promise<T>) {
  return runCanvasSessionTransition(async () => {
    let reopenMutations: (() => void) | undefined;
    try {
      await flushSpaceSessionCommits(async () => {
        reopenMutations = closeSpaceMutationAdmission(context.history);
        await waitForSpaceMutations(context.history);
      });
      await nextTick();
      context.captureOverview();
      return await transition();
    } finally {
      reopenMutations?.();
    }
  });
}

export function commitActiveSpaceSession(context: SpaceSessionContext) {
  return transitionActiveSpaceSession(context, async () => {});
}

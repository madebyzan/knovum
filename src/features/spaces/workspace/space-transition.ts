import { nextTick } from "vue";

import {
  flushSpaceSessionContent,
  flushSpaceSessionLayout,
  flushSpaceSessionPersistence,
  runCanvasSessionTransition,
} from "../canvas/editor-session";
import { closeSpaceMutationAdmission, waitForSpaceMutations } from "../history/mutation-barrier";
import type { SpaceSessionContext } from "./space-session-context";

export function transitionActiveSpaceSession<T>(context: SpaceSessionContext, transition: () => Promise<T>) {
  return runCanvasSessionTransition(async () => {
    await flushSpaceSessionContent();
    const reopenMutations = closeSpaceMutationAdmission(context.history);
    try {
      await waitForSpaceMutations(context.history);
      await flushSpaceSessionLayout();
      await flushSpaceSessionPersistence();
      await nextTick();
      context.captureOverview();
      return await transition();
    } finally {
      reopenMutations();
    }
  });
}

export function commitActiveSpaceSession(context: SpaceSessionContext) {
  return transitionActiveSpaceSession(context, async () => {});
}

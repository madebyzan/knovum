import { nextTick } from "vue";

import { runCanvasSessionTransition } from "../canvas/editor-session";
import { closeSpaceMutationAdmission, waitForSpaceMutations } from "../history/mutation-barrier";
import { flushSpaceSessionCommits } from "./session-participants";
import type { SpaceSessionContext } from "./space-session-context";
import type { SpaceSurface } from "./space-surface";

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

export function transitionSpaceSurface(context: SpaceSessionContext, surface: SpaceSurface) {
  const spaceId = context.activeSpaceId.value;
  assertEditorTarget(context, surface);
  return transitionActiveSpaceSession(context, async () => {
    if (context.activeSpaceId.value !== spaceId) throw new Error("Editor target Space changed");
    assertEditorTarget(context, surface);
    context.surface.value = surface;
    await nextTick();
  });
}

function assertEditorTarget(context: SpaceSessionContext, surface: SpaceSurface) {
  if (surface.kind !== "document" && surface.kind !== "video") return;
  const node = context.nodes.value.find(({ id }) => id === surface.nodeId);
  const eligible =
    surface.kind === "document"
      ? node?.data.kind === "note" || node?.data.kind === "markdown"
      : node?.data.kind === "video";
  if (!context.activeSpaceId.value || !eligible) throw new Error("Editor content is unavailable");
}

import { defineStore } from "pinia";
import { watch } from "vue";

import { createSpacePlacement } from "../canvas/navigation-requests";
import { useSpaceActions } from "./actions/use-space-actions";
import { createSpaceState, spaceSessionBindings } from "./session-state";

export const useSpaceStore = defineStore("spaces", () => {
  const state = createSpaceState();
  const placement = createSpacePlacement({
    insertionPoint: state.insertionPoint,
    revealRequest: state.revealRequest,
    viewportRequest: state.viewportRequest,
    zoomRequest: state.zoomRequest,
    selectionClearNonce: state.selectionClearNonce,
    sceneCue: state.sceneCue,
  });
  watch([state.nodes, state.surface], ([nodes, surface]) => {
    if (
      (surface.kind === "document" || surface.kind === "video") &&
      !nodes.some(({ id }) => id === surface.nodeId)
    )
      state.surface.value = { kind: "canvas" };
  });
  watch(
    state.surface,
    () => {
      state.sourcePeek.value = undefined;
      if (state.rightSidebarView.value === "source")
        state.rightSidebarView.value = state.sourceReturnView.value;
      state.editorLocateRequest.value = undefined;
      state.workflowMode.value = undefined;
      state.findOpen.value = false;
      state.globalFindOpen.value = false;
    },
    { flush: "sync" },
  );
  watch([state.nodes, state.relations], () => {
    state.thumbnailDirty.value = true;
  });
  return {
    ...spaceSessionBindings(state),
    ...placement,
    ...useSpaceActions(state, placement),
    markCanvasOverviewDirty: () => {
      state.thumbnailDirty.value = true;
    },
  };
});

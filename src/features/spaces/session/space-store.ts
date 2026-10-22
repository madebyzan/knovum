import { defineStore } from "pinia";
import { watch } from "vue";

import { createSpacePlacement } from "../canvas/navigation-requests";
import type { SpacePersistenceState } from "../workspace/persistence-state";
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
  watch(
    state.surface,
    () => {
      state.sourcePeek.value = undefined;
      if (state.rightSidebarView.value === "source")
        state.rightSidebarView.value = state.sourceReturnView.value;
      state.editorLocateRequest.value = undefined;
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
    /** An unsaved Space has no database row, so nothing may be written for it until it is created. */
    spacePersistence: (spaceId: string): SpacePersistenceState =>
      state.draftSpace.value?.id === spaceId ? "draft" : "persisted",
  };
});

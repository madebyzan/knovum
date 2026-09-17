import { defineStore } from "pinia";
import { watch } from "vue";

import { createSpacePlacement } from "../canvas/navigation-requests";
import { createSpaceActions } from "./actions/create-space-actions";
import { createSpaceState, publicSpaceState } from "./session-state";

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
  watch([state.nodes, state.focusedEditorId], ([nodes, focusedEditorId]) => {
    if (focusedEditorId && !nodes.some((node) => node.id === focusedEditorId))
      state.focusedEditorId.value = undefined;
  });
  watch([state.nodes, state.relations], () => {
    state.thumbnailDirty.value = true;
  });
  return {
    ...publicSpaceState(state),
    ...placement,
    ...createSpaceActions(state, placement),
    markCanvasOverviewDirty: () => {
      state.thumbnailDirty.value = true;
    },
  };
});

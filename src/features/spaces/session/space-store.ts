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
  watch([state.nodes, state.focusedEditorId], ([nodes, focusedEditorId]) => {
    if (focusedEditorId && !nodes.some((node) => node.id === focusedEditorId))
      state.focusedEditorId.value = undefined;
  });
  watch(state.nodes, (nodes) => {
    if (state.videoEditorNodeId.value && !nodes.some(({ id }) => id === state.videoEditorNodeId.value))
      state.videoEditorNodeId.value = undefined;
  });
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
    openVideoEditor: (nodeId: string) => {
      state.videoEditorNodeId.value = nodeId;
    },
    closeVideoEditor: () => {
      state.videoEditorNodeId.value = undefined;
    },
  };
});

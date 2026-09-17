import type { SpacePlacement } from "../../canvas/navigation-requests";
import { createInspectorActions } from "../../inspector/inspector-actions";
import { createSpaceMediaActions } from "../../media/actions";
import { createSelectionActions } from "../../organization/selection-actions";
import { materializeActiveSpace } from "../../workspace/draft-space-actions";
import type { SpaceSessionContext } from "../../workspace/space-session-actions";
import type { SpaceState } from "../session-state";
import { assembleSpaceActions } from "./assemble-space-actions";
import { createCanvasGestureActions } from "./canvas-gesture-actions";
import { createJsonCanvasImportActions } from "./json-canvas-import-actions";
import { createSpaceNodeLifecycleActions } from "./node-lifecycle-actions";
import { createSpaceMutations } from "./node-mutations";
import { createPlacementActions } from "./placement-actions";
import { removeSpaceRelations, type RelationContext } from "./relation-actions";
import { createSpaceViewActions } from "./view-actions";

export function createSpaceActions(state: SpaceState, placement: SpacePlacement) {
  const spaceContext = createSpaceContext(state);
  const mutations = createSpaceMutations({
    nodes: state.nodes,
    activeSpaceId: state.activeSpaceId,
    history: state.history,
    nextInsertionPoint: placement.nextInsertionPoint,
    ensureActiveSpacePersisted: () => materializeActiveSpace(spaceContext),
  });
  const placementActions = createPlacementActions(mutations, placement, state.activeSpaceId);
  const relation: RelationContext = {
    activeSpaceId: state.activeSpaceId,
    relations: state.relations,
    history: state.history,
  };
  const organization = createSelectionActions({
    nodes: state.nodes,
    ...mutations,
    addItem: placementActions.addItem,
    removeRelations: (ids) => removeSpaceRelations(relation, ids),
  });
  const media = createSpaceMediaActions({
    nodes: state.nodes,
    analysisIds: state.mediaAnalysisIds,
    analysisPreviews: state.mediaAnalysisPreviews,
    seekRequest: state.mediaSeekRequest,
    toggleRequest: state.mediaToggleRequest,
    playback: state.mediaPlayback,
    saveNodeData: mutations.saveNodeData,
  });
  const lifecycle = createSpaceNodeLifecycleActions({
    activeSpaceId: state.activeSpaceId,
    nodes: state.nodes,
    relations: state.relations,
  });
  const context = createInspectorActions(state, placement, media.requestMediaSeek);
  const view = createSpaceViewActions(state, placement);
  const actions = assembleSpaceActions({
    state,
    mutations,
    relation,
    placementActions,
    organization,
    media,
    lifecycle,
    context,
    spaceContext,
    view,
  });
  const canvasImport = createJsonCanvasImportActions({
    ensureActiveSpacePersisted: () => materializeActiveSpace(spaceContext),
    mutations,
    placement,
    state,
  });
  return {
    ...actions,
    ensureActiveSpacePersisted: () => materializeActiveSpace(spaceContext),
    ...canvasImport,
    ...createCanvasGestureActions(state),
  };
}

function createSpaceContext(state: SpaceState): SpaceSessionContext {
  return {
    spaces: state.spaces,
    draftSpace: state.draftSpace,
    activeSpaceId: state.activeSpaceId,
    nodes: state.nodes,
    scenes: state.scenes,
    relations: state.relations,
    ready: state.ready,
    loading: state.loading,
    focusedEditorId: state.focusedEditorId,
    history: state.history,
    runtime: { openGeneration: 0 },
    openSpaceIds: state.openSpaceIds,
    closedSpaceIds: state.closedSpaceIds,
    navigationHistory: state.navigationHistory,
    navigationIndex: state.navigationIndex,
  };
}

import type { SpacePlacement } from "../../canvas/navigation-requests";
import { createWorkflowPanelActions } from "../../canvas/workflow-panel-actions";
import { observeEnrichmentJobs } from "../../enrichment/job-observation";
import { createSpaceMediaActions } from "../../media/actions";
import { createSpaceAudioRecordingActions } from "../../media/audio-recording-actions";
import { createRightSidebarActions } from "../../right-sidebar/right-sidebar-actions";
import { materializeActiveSpace } from "../../workspace/draft-space-actions";
import type { SpaceSessionContext } from "../../workspace/space-session-context";
import type { SpaceState } from "../session-state";
import { assembleSpaceActions } from "./assemble-space-actions";
import { createJsonCanvasImportActions } from "./json-canvas-import-actions";
import { createSpaceMutations } from "./node-mutations";
import { createPlacementActions } from "./placement-actions";
import type { RelationContext } from "./relation-actions";
import { createStructureActions } from "./structure-actions";
import { createStructureTransferActions } from "./structure-transfer-actions";
import { createSpaceViewActions } from "./view-actions";

export function createSpaceActions(state: SpaceState, placement: SpacePlacement) {
  const spaceContext = createSpaceContext(state);
  const structure = createStructureActions({
    activeSpaceId: state.activeSpaceId,
    editSnapshot: state.editSnapshot,
    nodes: state.nodes,
    relations: state.relations,
    history: state.history,
    ensureActiveSpacePersisted: () => materializeActiveSpace(spaceContext),
  });
  const mutations = createSpaceMutations({
    nodes: state.nodes,
    editSnapshot: state.editSnapshot,
    groupScopePath: state.groupScopePath,
    activeSpaceId: state.activeSpaceId,
    history: state.history,
    nextInsertionPoint: placement.nextInsertionPoint,
    ensureActiveSpacePersisted: () => materializeActiveSpace(spaceContext),
    recordCommittedSpaceEdit: structure.recordCommittedSpaceEdit,
  });
  const placementActions = createPlacementActions(mutations, placement, state.activeSpaceId);
  const relation: RelationContext = {
    activeSpaceId: state.activeSpaceId,
    relations: state.relations,
    history: state.history,
  };
  observeEnrichmentJobs({
    activeSpaceId: state.activeSpaceId,
    analysisIds: state.mediaAnalysisIds,
    analysisPreviews: state.mediaAnalysisPreviews,
    applyNodeContentForSpace: mutations.applyNodeContentForSpace,
  });
  const media = createSpaceMediaActions({
    activeSpaceId: state.activeSpaceId,
    applyNodeContentForSpace: mutations.applyNodeContentForSpace,
    nodes: state.nodes,
    analysisIds: state.mediaAnalysisIds,
    analysisPreviews: state.mediaAnalysisPreviews,
    seekRequest: state.mediaSeekRequest,
    toggleRequest: state.mediaToggleRequest,
    playback: state.mediaPlayback,
  });
  const audioRecording = createSpaceAudioRecordingActions({
    activeSpaceId: state.activeSpaceId,
    target: state.audioRecordingTarget,
    nextInsertionPoint: placement.nextInsertionPoint,
    ensureActiveSpacePersisted: () => materializeActiveSpace(spaceContext),
    createPendingContentForSpace: mutations.createPendingContentForSpace,
    finalizePendingContent: placementActions.finalizePendingContent,
  });
  const structureTransfer = createStructureTransferActions({
    activeSpaceId: state.activeSpaceId,
    editSnapshot: state.editSnapshot,
    nodes: state.nodes,
    relations: state.relations,
    recordCommittedSpaceEdit: structure.recordCommittedSpaceEdit,
  });
  const context = createRightSidebarActions(state, placement, media.requestMediaSeek);
  const view = createSpaceViewActions(state, placement);
  const actions = assembleSpaceActions({
    state,
    mutations,
    relation,
    placementActions,
    media,
    structureTransfer,
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
    ...createWorkflowPanelActions(state),
    ...audioRecording,
    ensureActiveSpacePersisted: () => materializeActiveSpace(spaceContext),
    ...canvasImport,
    ...structure,
  };
}

function createSpaceContext(state: SpaceState): SpaceSessionContext {
  return {
    spaces: state.spaces,
    draftSpace: state.draftSpace,
    activeSpaceId: state.activeSpaceId,
    nodes: state.nodes,
    editSnapshot: state.editSnapshot,
    groupScopePath: state.groupScopePath,
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

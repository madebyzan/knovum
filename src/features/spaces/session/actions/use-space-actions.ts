import type { SpacePlacement } from "../../canvas/navigation-requests";
import { createWorkflowPanelActions } from "../../canvas/workflow-panel-actions";
import { useEnrichmentJobObservation } from "../../enrichment/use-enrichment-job-observation";
import { createSpaceMediaActions } from "../../media/actions";
import { createSpaceAudioRecordingActions } from "../../media/audio-recording-actions";
import { materializeActiveSpace } from "../../workspace/draft-space-actions";
import { createEditorSessions } from "../../workspace/editor-sessions";
import { createEditorSurfaceActions } from "../../workspace/editor-surface-actions";
import { createRightSidebarActions } from "../../workspace/right-sidebar-actions";
import { useEditorSessionProjection } from "../../workspace/use-editor-session-projection";
import { closeWorkbenchTab } from "../../workspace/workbench-close-actions";
import type { SpaceState } from "../session-state";
import { createSpaceSessionContext } from "../space-session-composition";
import { assembleSpaceActions } from "./assemble-space-actions";
import { createJsonCanvasImportActions } from "./json-canvas-import-actions";
import { createSpaceMutations } from "./node-mutations";
import { createPlacementActions } from "./placement-actions";
import type { RelationContext } from "./relation-actions";
import { createStructureActions } from "./structure-actions";
import { createStructureTransferActions } from "./structure-transfer-actions";
import { createSpaceViewActions } from "./view-actions";

export function useSpaceActions(state: SpaceState, placement: SpacePlacement) {
  const spaceContext = createSpaceSessionContext(state, createEditorSessions());
  // Shared by every action group below: a single stable reference instead of
  // a fresh closure per call site, so there is exactly one place that knows
  // how "ensure the active space is persisted" is implemented.
  const ensureActiveSpacePersisted = () => materializeActiveSpace(spaceContext);
  const structure = createStructureActions({
    activeSpaceId: state.activeSpaceId,
    editSnapshot: state.editSnapshot,
    nodes: state.nodes,
    relations: state.relations,
    history: state.history,
    ensureActiveSpacePersisted,
    editorTabs: state.editorTabs,
  });
  const mutations = createSpaceMutations({
    nodes: state.nodes,
    editSnapshot: state.editSnapshot,
    groupScopePath: state.groupScopePath,
    activeSpaceId: state.activeSpaceId,
    history: state.history,
    nextInsertionPoint: placement.nextInsertionPoint,
    ensureActiveSpacePersisted,
    recordCommittedSpaceEdit: structure.recordCommittedSpaceEdit,
  });
  const placementActions = createPlacementActions(mutations, placement, state.activeSpaceId);
  const relation: RelationContext = {
    activeSpaceId: state.activeSpaceId,
    relations: state.relations,
    history: state.history,
  };
  useEnrichmentJobObservation({
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
    ensureActiveSpacePersisted,
    createPendingContentForSpace: mutations.createPendingContentForSpace,
    finalizePendingContent: placementActions.finalizePendingContent,
  });
  const structureTransfer = createStructureTransferActions({
    activeSpaceId: state.activeSpaceId,
    editSnapshot: state.editSnapshot,
    nodes: state.nodes,
    relations: state.relations,
    recordCommittedSpaceEdit: structure.recordCommittedSpaceEdit,
    editorTabs: state.editorTabs,
    history: state.history,
  });
  const editors = createEditorSurfaceActions(spaceContext);
  useEditorSessionProjection(state, (tabId) => closeWorkbenchTab(spaceContext, tabId));
  const context = createRightSidebarActions(
    state,
    placement,
    media.requestMediaSeek,
    editors.openDocumentEditor,
  );
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
    ensureActiveSpacePersisted,
    mutations,
    placement,
    state,
  });
  return {
    ...actions,
    ...editors,
    ...createWorkflowPanelActions(state),
    ...audioRecording,
    ensureActiveSpacePersisted,
    ...canvasImport,
    ...structure,
  };
}

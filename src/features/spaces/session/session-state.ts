import { createSpaceHistoryState } from "../history/history-state";
import { createSpaceMediaState } from "../media/media-state";
import { createWorkspaceNavigationState } from "../workspace/navigation-history";
import { createSpaceContentState } from "./content-state";
import { createSpaceUiState } from "./workbench-state";

export function createSpaceState() {
  const ui = createSpaceUiState();
  return {
    ...createSpaceContentState(),
    ...ui,
    ...createSpaceMediaState(),
    ...createSpaceHistoryState(),
    ...createWorkspaceNavigationState(),
  };
}

export type SpaceState = ReturnType<typeof createSpaceState>;

export function spaceSessionBindings(state: SpaceState) {
  return {
    spaces: state.spaces,
    draftSpace: state.draftSpace,
    nodes: state.nodes,
    editSnapshot: state.editSnapshot,
    groupScopePath: state.groupScopePath,
    urlCaptures: state.urlCaptures,
    scenes: state.scenes,
    relations: state.relations,
    activeSpace: state.activeSpace,
    activeSpaceId: state.activeSpaceId,
    ready: state.ready,
    loading: state.loading,
    workspaceMode: state.workspaceMode,
    libraryScope: state.libraryScope,
    libraryLayout: state.libraryLayout,
    libraryQuery: state.libraryQuery,
    librarySearchNonce: state.librarySearchNonce,
    audioRecordingTarget: state.audioRecordingTarget,
    findOpen: state.findOpen,
    globalFindOpen: state.globalFindOpen,
    askScopeRequest: state.askScopeRequest,
    rightSidebarView: state.rightSidebarView,
    workflowMode: state.workflowMode,
    workflowCaptureSelection: state.workflowCaptureSelection,
    focusedEditorId: state.focusedEditorId,
    videoEditorNodeId: state.videoEditorNodeId,
    insertionPoint: state.insertionPoint,
    revealRequest: state.revealRequest,
    viewportRequest: state.viewportRequest,
    zoomRequest: state.zoomRequest,
    selectionClearNonce: state.selectionClearNonce,
    sceneCue: state.sceneCue,
    mediaAnalysisIds: state.mediaAnalysisIds,
    mediaAnalysisPreviews: state.mediaAnalysisPreviews,
    mediaSeekRequest: state.mediaSeekRequest,
    mediaToggleRequest: state.mediaToggleRequest,
    mediaPlayback: state.mediaPlayback,
    sourcePeek: state.sourcePeek,
    sourceReturnView: state.sourceReturnView,
    editorLocateRequest: state.editorLocateRequest,
    canvasFocusIds: state.canvasFocusIds,
    outlineFocusRequest: state.outlineFocusRequest,
    outlineRenameStructureNodeId: state.outlineRenameStructureNodeId,
    recentlyDeletedSpaceId: state.recentlyDeletedSpaceId,
    relationKindFilter: state.relationKindFilter,
    relationStatusFilter: state.relationStatusFilter,
    savedQueryFilter: state.savedQueryFilter,
    canUndo: state.canUndo,
    canRedo: state.canRedo,
    openSpaceIds: state.openSpaceIds,
    closedSpaceIds: state.closedSpaceIds,
    canNavigateBack: state.canNavigateBack,
    canNavigateForward: state.canNavigateForward,
  };
}

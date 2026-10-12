import { computed } from "vue";

import { createSpaceHistoryState } from "../history/history-state";
import { createSpaceMediaState } from "../media/media-state";
import { createWorkbenchNavigationState } from "../workspace/navigation-history";
import { spaceWorkbenchCapabilities } from "../workspace/space-surface";
import { createSpaceWorkbenchTabs } from "../workspace/workbench-tab-state";
import { createSpaceContentState } from "./content-state";
import { createSpaceUiState } from "./workbench-state";

export function createSpaceState() {
  const ui = createSpaceUiState();
  const content = createSpaceContentState();
  const workbench = createSpaceWorkbenchTabs(content);
  return {
    ...content,
    ...workbench,
    ...ui,
    workbench: computed(() =>
      spaceWorkbenchCapabilities(workbench.surface.value, Boolean(workbench.activeSpaceId.value)),
    ),
    ...createSpaceMediaState(),
    ...createSpaceHistoryState(),
    ...createWorkbenchNavigationState(),
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
    surface: state.surface,
    tabs: state.tabs,
    activeTabId: state.activeTabId,
    editorTabs: state.editorTabs,
    closedTabs: state.closedTabs,
    permanentTabId: state.permanentTabId,
    workbench: state.workbench,
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
    canNavigateBack: state.canNavigateBack,
    canNavigateForward: state.canNavigateForward,
  };
}

import { captureActiveCanvasThumbnail } from "../canvas/thumbnail-scheduler";
import type { SpaceSessionContext } from "../workspace/space-session-context";
import type { SpaceState } from "./session-state";

/** Composes the session context that Space and workbench actions operate on. */
export function createSpaceSessionContext(
  state: SpaceState,
  editors: SpaceSessionContext["editors"],
): SpaceSessionContext {
  return {
    captureOverview: () => captureActiveCanvasThumbnail(state),
    spaces: state.spaces,
    draftSpace: state.draftSpace,
    nodes: state.nodes,
    editSnapshot: state.editSnapshot,
    groupScopePath: state.groupScopePath,
    scenes: state.scenes,
    relations: state.relations,
    ready: state.ready,
    loading: state.loading,
    tabs: state.tabs,
    activeTabId: state.activeTabId,
    closedTabs: state.closedTabs,
    workspaceDestination: state.workspaceDestination,
    activeTab: state.activeTab,
    activeSpaceId: state.activeSpaceId,
    activeSpace: state.activeSpace,
    editorTabs: state.editorTabs,
    permanentTabId: state.permanentTabId,
    surface: state.surface,
    editors,
    history: state.history,
    runtime: { openGeneration: 0, historyBySpace: new Map() },
    navigationHistory: state.navigationHistory,
    navigationIndex: state.navigationIndex,
  };
}

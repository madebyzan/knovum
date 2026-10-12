import type { SpaceHistoryState } from "../../history/stack";
import type { Space } from "../../model/space";
import { createSpaceState } from "../../session/session-state";
import { createSpaceSessionContext } from "../../session/space-session-composition";
import type { SpaceSessionContext } from "../../workspace/space-session-context";
import {
  createSpaceCanvasTab,
  createSpaceEditorTab,
  type SpaceEditorTab,
  type SpaceWorkbenchTab,
} from "../../workspace/workbench-tab";
import {
  insertWorkbenchTab,
  selectWorkbenchTab,
  type SpaceWorkbenchTabState,
} from "../../workspace/workbench-tab-state";

export function createTestSpace(overrides: Partial<Space> = {}): Space {
  return {
    id: "inbox",
    kind: "inbox",
    title: "Inbox",
    archivedAt: null,
    favorite: false,
    thumbnailPath: null,
    nodeCount: 0,
    lastOpenedAt: 1,
    createdAt: 1,
    updatedAt: 1,
    ...overrides,
  };
}

interface TestSessionInput {
  activeSpaceId?: string;
  spaces?: Space[];
  editors?: SpaceSessionContext["editors"];
  history?: SpaceHistoryState;
  captureOverview?: () => void;
}

/**
 * Builds the real session state with the canvas tab of `activeSpaceId` already open, which is the
 * state the application reaches right after initialization.
 */
export function createTestSessionState(input: TestSessionInput = {}) {
  const state = createSpaceState();
  state.spaces.value = input.spaces ?? [createTestSpace()];
  const context = createSpaceSessionContext(
    state,
    input.editors ?? { open: () => {}, release: () => {}, canDiscard: () => false },
  );
  // Thumbnail capture reaches the desktop host, which no renderer test owns.
  context.captureOverview = input.captureOverview ?? (() => undefined);
  if (input.history) context.history = input.history;
  const tab = openTestSpaceTab(context, input.activeSpaceId ?? "inbox");
  context.navigationHistory.value = [tab.id];
  context.navigationIndex.value = 0;
  return { state, context };
}

export function createTestSessionContext(input: TestSessionInput = {}) {
  return createTestSessionState(input).context;
}

export function openTestSpaceTab(state: SpaceWorkbenchTabState, spaceId: string) {
  const tab = createSpaceCanvasTab(spaceId);
  insertWorkbenchTab(state, tab);
  selectWorkbenchTab(state, tab.id);
  return tab;
}

/** Seeds a store-backed test with the canvas tab of one Space, without loading a projection. */
export function openStoreSpaceTab(
  store: { tabs: readonly SpaceWorkbenchTab[]; activeTabId: string | undefined },
  spaceId: string,
) {
  const tab = createSpaceCanvasTab(spaceId);
  store.tabs = [...store.tabs, tab];
  store.activeTabId = tab.id;
  return tab;
}

export function activateTestEditor(
  context: SpaceSessionContext,
  kind: SpaceEditorTab["kind"],
  nodeId: string,
) {
  const tab = createSpaceEditorTab(context.activeSpaceId.value!, kind, nodeId);
  insertWorkbenchTab(context, tab);
  selectWorkbenchTab(context, tab.id);
  return tab;
}

import { computed, shallowRef, type Ref } from "vue";

import type { Space } from "../model/space";
import type { SpaceSurface } from "./space-surface";
import {
  findSpaceCanvasTab,
  isSpaceEditorTab,
  tabInsertionIndex,
  workbenchTabSurface,
  type SpaceWorkbenchTab,
} from "./workbench-tab";

export interface ClosedWorkbenchTab {
  readonly tab: SpaceWorkbenchTab;
  readonly index: number;
}

interface WorkbenchSpaceSource {
  spaces: Ref<Space[]>;
  draftSpace: Ref<Space | undefined>;
}

/**
 * The workbench owns one ordered tab list and one active tab. The active Space, the work surface
 * and editor tab projections derive from it; nothing else stores a parallel selection.
 */
export function createSpaceWorkbenchTabs(source: WorkbenchSpaceSource) {
  const tabs = shallowRef<readonly SpaceWorkbenchTab[]>([]);
  const activeTabId = shallowRef<string>();
  const closedTabs = shallowRef<readonly ClosedWorkbenchTab[]>([]);
  const workspaceDestination = shallowRef<"workbench" | "library">("workbench");
  const activeTab = computed(() => tabs.value.find(({ id }) => id === activeTabId.value));
  const activeSpaceId = computed(() => activeTab.value?.spaceId);
  const activeSpace = computed(() => {
    const draft = source.draftSpace.value;
    if (draft?.id === activeSpaceId.value) return draft;
    return source.spaces.value.find((space) => space.id === activeSpaceId.value);
  });
  const editorTabs = computed(() => tabs.value.filter(isSpaceEditorTab));
  const permanentTabId = computed(() => {
    const inbox = source.spaces.value.find((space) => space.kind === "inbox");
    return inbox ? findSpaceCanvasTab(tabs.value, inbox.id)?.id : undefined;
  });
  const surface = computed<SpaceSurface>(() => {
    if (workspaceDestination.value === "library") return { kind: "library" };
    const tab = activeTab.value;
    return tab ? workbenchTabSurface(tab) : { kind: "canvas" };
  });
  return {
    tabs,
    activeTabId,
    closedTabs,
    workspaceDestination,
    activeTab,
    activeSpaceId,
    activeSpace,
    editorTabs,
    permanentTabId,
    surface,
  };
}

export type SpaceWorkbenchTabState = ReturnType<typeof createSpaceWorkbenchTabs>;

export function requireWorkbenchTab(state: SpaceWorkbenchTabState, tabId: string) {
  const tab = state.tabs.value.find(({ id }) => id === tabId);
  if (!tab) throw new Error(`Unknown workbench tab: ${tabId}`);
  return tab;
}

export function selectWorkbenchTab(state: SpaceWorkbenchTabState, tabId: string) {
  requireWorkbenchTab(state, tabId);
  state.activeTabId.value = tabId;
  state.workspaceDestination.value = "workbench";
}

export function insertWorkbenchTab(
  state: SpaceWorkbenchTabState,
  tab: SpaceWorkbenchTab,
  index = tabInsertionIndex(state.tabs.value, state.activeTabId.value),
) {
  const next = [...state.tabs.value];
  next.splice(Math.min(index, next.length), 0, tab);
  state.tabs.value = next;
}

/** Removes closed tabs and remembers them for reopening at their original position. */
export function closeWorkbenchTabs(state: SpaceWorkbenchTabState, closing: readonly SpaceWorkbenchTab[]) {
  const closed = closing.map((tab) => ({
    tab,
    index: state.tabs.value.findIndex(({ id }) => id === tab.id),
  }));
  discardWorkbenchTabs(state, closing);
  state.closedTabs.value = [...closed.toReversed(), ...state.closedTabs.value];
}

/** Removes tabs that must not come back, such as a discarded draft or an unavailable resource. */
export function discardWorkbenchTabs(state: SpaceWorkbenchTabState, discarded: readonly SpaceWorkbenchTab[]) {
  const removed = new Set(discarded.map(({ id }) => id));
  if (state.activeTabId.value && removed.has(state.activeTabId.value))
    throw new Error("Another workbench tab must be activated before the active tab is removed");
  state.tabs.value = state.tabs.value.filter(({ id }) => !removed.has(id));
  state.closedTabs.value = state.closedTabs.value.filter(({ tab }) => !removed.has(tab.id));
}

export function takeClosedWorkbenchTab(state: SpaceWorkbenchTabState) {
  const [closed, ...remaining] = state.closedTabs.value;
  if (closed) state.closedTabs.value = remaining;
  return closed;
}

/** Drops every trace of a Space that left the product, such as an archived or deleted Space. */
export function discardSpaceWorkbenchTabs(state: SpaceWorkbenchTabState, spaceId: string) {
  const discarded = state.tabs.value.filter((tab) => tab.spaceId === spaceId);
  discardWorkbenchTabs(state, discarded);
  state.closedTabs.value = state.closedTabs.value.filter(({ tab }) => tab.spaceId !== spaceId);
  return discarded;
}

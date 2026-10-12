import { nextTick } from "vue";

import { ensureDraftSpace } from "./draft-space-actions";
import { recordWorkbenchNavigation } from "./navigation-history";
import { createAndTrackSpace } from "./space-creation";
import { listSpaces } from "./space-operations";
import {
  cancelPendingSpaceOpen,
  requireAvailableSpace,
  startSpaceSessionLoad,
} from "./space-session-actions";
import type { SpaceSessionContext } from "./space-session-context";
import { transitionActiveSpaceSession } from "./space-transition";
import {
  createSpaceCanvasTab,
  createSpaceEditorTab,
  findEditorTab,
  findSpaceCanvasTab,
  isSpaceEditorTab,
  type SpaceEditorTab,
  type SpaceWorkbenchTab,
} from "./workbench-tab";
import {
  insertWorkbenchTab,
  requireWorkbenchTab,
  selectWorkbenchTab,
  takeClosedWorkbenchTab,
} from "./workbench-tab-state";

export async function initializeSpaces(context: SpaceSessionContext, inboxTitle: string) {
  if (context.ready.value) return;
  context.spaces.value = await listSpaces();
  const inbox =
    context.spaces.value.find((space) => space.kind === "inbox") ??
    (await createAndTrackSpace(context, inboxTitle, "inbox"));
  await openSpace(context, inbox.id);
  context.ready.value = true;
}

export async function createSpace(context: SpaceSessionContext, title: string) {
  await openSpace(context, ensureDraftSpace(context, title).id);
}

/** Focuses the canvas tab of a Space, opening one right of the active tab when it has none. */
export async function openSpace(context: SpaceSessionContext, spaceId: string) {
  const existing = findSpaceCanvasTab(context.tabs.value, spaceId);
  if (existing) return activateWorkbenchTab(context, existing.id);
  requireAvailableSpace(context, spaceId);
  return enterWorkbenchTab(context, createSpaceCanvasTab(spaceId), {});
}

export async function openSpaceCanvas(context: SpaceSessionContext) {
  return openSpace(context, requireActiveSpaceId(context));
}

export async function openEditorTab(
  context: SpaceSessionContext,
  kind: SpaceEditorTab["kind"],
  nodeId: string,
) {
  const spaceId = requireActiveSpaceId(context);
  const existing = findEditorTab(context.tabs.value, { spaceId, kind, nodeId });
  if (existing) return activateWorkbenchTab(context, existing.id);
  return enterWorkbenchTab(context, createSpaceEditorTab(spaceId, kind, nodeId), {});
}

export async function activateWorkbenchTab(context: SpaceSessionContext, tabId: string) {
  const tab = requireWorkbenchTab(context, tabId);
  if (!(await enterWorkbenchTab(context, tab))) return false;
  recordWorkbenchNavigation(context, tab.id);
  return true;
}

export async function reopenClosedWorkbenchTab(context: SpaceSessionContext) {
  const closed = context.closedTabs.value[0];
  if (!closed) return false;
  requireAvailableSpace(context, closed.tab.spaceId);
  if (!(await enterWorkbenchTab(context, closed.tab, { index: closed.index }))) return false;
  takeClosedWorkbenchTab(context);
  recordWorkbenchNavigation(context, closed.tab.id);
  return true;
}

export async function navigateWorkbenchHistory(context: SpaceSessionContext, direction: -1 | 1) {
  const nextIndex = context.navigationIndex.value + direction;
  const tabId = context.navigationHistory.value[nextIndex];
  if (!tabId) return;
  if (!(await enterWorkbenchTab(context, requireWorkbenchTab(context, tabId)))) return;
  context.navigationIndex.value = nextIndex;
}

/**
 * Activates one tab, loading its Space session first when the tab belongs to another Space.
 * Passing an insertion adds the tab to the strip as part of the same transition; omitting it
 * activates a tab that is already open without recording navigation.
 */
export async function enterWorkbenchTab(
  context: SpaceSessionContext,
  tab: SpaceWorkbenchTab,
  insertion?: { index?: number },
) {
  if (!insertion && context.activeTabId.value === tab.id) {
    cancelPendingSpaceOpen(context);
    selectWorkbenchTab(context, tab.id);
    return true;
  }
  const needsSession = tab.spaceId !== context.activeSpaceId.value;
  if (needsSession && context.runtime.pendingSpaceId === tab.spaceId) return false;
  const load = needsSession
    ? startSpaceSessionLoad(context, tab.spaceId, { markOpened: true, resetHistory: true })
    : undefined;
  return transitionActiveSpaceSession(context, async () => {
    if (load) {
      const applySession = await load.complete();
      if (!applySession) return false;
      applySession();
    }
    if (insertion) {
      if (isSpaceEditorTab(tab)) openEditorSession(context, tab);
      insertWorkbenchTab(context, tab, insertion.index);
    }
    selectWorkbenchTab(context, tab.id);
    await nextTick();
    return true;
  });
}

function openEditorSession(context: SpaceSessionContext, tab: SpaceEditorTab) {
  const node = context.nodes.value.find(({ id }) => id === tab.nodeId);
  if (!node) throw new Error("Editor content is unavailable");
  context.editors.open(tab, node);
}

function requireActiveSpaceId(context: SpaceSessionContext) {
  const spaceId = context.activeSpaceId.value;
  if (!spaceId) throw new Error("Space requires an active space");
  return spaceId;
}

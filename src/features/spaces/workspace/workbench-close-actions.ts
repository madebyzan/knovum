import { runCanvasSessionTransition } from "../canvas/editor-session";
import { releaseEditorTabs } from "./editor-tab-lifecycle";
import { removeWorkbenchNavigation } from "./navigation-history";
import { prepareEditorSessionClose } from "./session-participants";
import { releaseUnreferencedSpaces } from "./space-session-actions";
import type { SpaceSessionContext } from "./space-session-context";
import { enterWorkbenchTab } from "./workbench-navigation-actions";
import {
  isSpaceEditorTab,
  referencedSpaceIds,
  successorTabId,
  type SpaceWorkbenchTab,
} from "./workbench-tab";
import {
  closeWorkbenchTabs,
  discardSpaceWorkbenchTabs,
  discardWorkbenchTabs,
  requireWorkbenchTab,
  selectWorkbenchTab,
} from "./workbench-tab-state";

/** The Inbox canvas tab is permanent, so the workbench always retains at least one tab. */
export async function closeWorkbenchTab(context: SpaceSessionContext, tabId: string) {
  const tab = context.tabs.value.find(({ id }) => id === tabId);
  if (!tab || tab.id === context.permanentTabId.value) return;
  await closeTabs(context, [tab]);
}

export async function closeOtherWorkbenchTabs(context: SpaceSessionContext, tabId: string) {
  requireWorkbenchTab(context, tabId);
  await closeTabs(
    context,
    closableTabs(context, (tab) => tab.id !== tabId),
    tabId,
  );
}

export async function closeWorkbenchTabsToRight(context: SpaceSessionContext, tabId: string) {
  const index = context.tabs.value.findIndex(({ id }) => id === tabId);
  if (index < 0) return;
  const toRight = new Set(context.tabs.value.slice(index + 1).map(({ id }) => id));
  await closeTabs(
    context,
    closableTabs(context, (tab) => toRight.has(tab.id)),
    tabId,
  );
}

/** Removes one tab without asking for a save decision, so it cannot be reopened. */
export async function discardWorkbenchTab(context: SpaceSessionContext, tab: SpaceWorkbenchTab) {
  const successor = closingSuccessor(context, [tab]);
  await runCanvasSessionTransition(async () => {
    if (successor) selectWorkbenchTab(context, successor.id);
    const previousSpaceIds = referencedSpaceIds(context.tabs.value);
    discardWorkbenchTabs(context, [tab]);
    await finishTabRemoval(context, [tab], previousSpaceIds);
  });
}

/** Drops the tabs of a Space that left the product; its editors were already closed. */
export async function discardSpaceTabs(context: SpaceSessionContext, spaceId: string) {
  const previousSpaceIds = referencedSpaceIds(context.tabs.value);
  const discarded = discardSpaceWorkbenchTabs(context, spaceId);
  await finishTabRemoval(context, discarded, previousSpaceIds);
}

function closableTabs(context: SpaceSessionContext, selected: (tab: SpaceWorkbenchTab) => boolean) {
  return context.tabs.value.filter((tab) => tab.id !== context.permanentTabId.value && selected(tab));
}

async function closeTabs(
  context: SpaceSessionContext,
  closing: readonly SpaceWorkbenchTab[],
  activateTabId?: string,
) {
  if (!closing.length) return;
  const editors = closing.filter(isSpaceEditorTab);
  const successor = closingSuccessor(context, closing, activateTabId);
  const finalizeClose = await prepareEditorSessionClose({ tabIds: editors.map(({ id }) => id) });
  if (successor && successor.spaceId !== context.activeSpaceId.value) {
    // Leaving the active Space: its session switch owns the transition and loads the next Space.
    if (!(await enterWorkbenchTab(context, successor))) return;
    await removeClosedTabs(context, closing, finalizeClose);
    return;
  }
  // Staying inside the active Space: the save decisions above already covered the closing editors,
  // so only canvas interaction has to pause while their views unmount.
  await runCanvasSessionTransition(async () => {
    if (successor) selectWorkbenchTab(context, successor.id);
    await removeClosedTabs(context, closing, finalizeClose);
  });
}

async function removeClosedTabs(
  context: SpaceSessionContext,
  closing: readonly SpaceWorkbenchTab[],
  finalizeClose: () => void,
) {
  const previousSpaceIds = referencedSpaceIds(context.tabs.value);
  finalizeClose();
  closeWorkbenchTabs(context, closing);
  await finishTabRemoval(context, closing, previousSpaceIds);
}

/** Resolves the tab that stays active once the closing tabs leave the strip. */
function closingSuccessor(
  context: SpaceSessionContext,
  closing: readonly SpaceWorkbenchTab[],
  activateTabId?: string,
) {
  const closingIds = new Set(closing.map(({ id }) => id));
  const nextTabId =
    activateTabId ?? successorTabId(context.tabs.value, closingIds, context.activeTabId.value);
  if (!nextTabId) {
    if (context.activeTabId.value) throw new Error("The workbench must retain at least one tab");
    return undefined;
  }
  if (nextTabId === context.activeTabId.value) return undefined;
  return requireWorkbenchTab(context, nextTabId);
}

async function finishTabRemoval(
  context: SpaceSessionContext,
  removed: readonly SpaceWorkbenchTab[],
  previousSpaceIds: readonly string[],
) {
  removeWorkbenchNavigation(context, new Set(removed.map(({ id }) => id)));
  await releaseEditorTabs(context, removed.filter(isSpaceEditorTab));
  releaseUnreferencedSpaces(context, previousSpaceIds);
}

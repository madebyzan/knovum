import { nextTick } from "vue";

import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import type { Space } from "../model/space";
import { loadSpaceSessionProjection, type SpaceSessionContent } from "../session/space-projection";
import { acceptsEditorContent } from "./editor-sessions";
import { recordWorkbenchNavigation } from "./navigation-history";
import { retainSpaceSession } from "./retained-space-sessions";
import { markSpaceOpenedAndTrack } from "./space-creation";
import { activateSpaceSession } from "./space-session-actions";
import type { SpaceSessionContext } from "./space-session-context";
import { transitionActiveSpaceSession } from "./space-transition";
import { restoreSpaceViewSession } from "./space-view-session";
import { takeWorkbenchSession, type SavedWorkbenchTab, type WorkbenchSession } from "./workbench-session";
import {
  createSpaceCanvasTab,
  createSpaceEditorTab,
  referencedSpaceIds,
  type SpaceCanvasTab,
  type SpaceEditorTab,
  type SpaceWorkbenchTab,
} from "./workbench-tab";
import { selectWorkbenchTab } from "./workbench-tab-state";

type ReopenedTab = { tab: SpaceCanvasTab } | { tab: SpaceEditorTab; node: SpaceCanvasNode };

interface ReopenedStrip {
  entries: readonly ReopenedTab[];
  active: SpaceWorkbenchTab;
}

/**
 * Opens the workbench with the session the window last showed when startup restoration is enabled.
 * Every saved Space, editor tab and Space view context is revalidated against current persisted
 * content; items that left the Space drop out cleanly. The permanent Inbox tab is always present
 * and becomes active when restoration is disabled or the saved active tab did not return.
 */
export async function restoreWorkbench(
  context: SpaceSessionContext,
  inbox: Space,
  restorePreviousWorkspace = true,
) {
  const taken = takeWorkbenchSession();
  const saved = restorePreviousWorkspace ? taken : undefined;
  const available = new Set(
    context.spaces.value.filter(({ archivedAt }) => archivedAt === null).map(({ id }) => id),
  );
  const candidates = (saved?.tabs ?? []).filter(({ spaceId }) => available.has(spaceId));
  const contents = await loadSpaceContents([inbox.id, ...candidates.map(({ spaceId }) => spaceId)]);
  const strip = reopenStrip(saved, candidates, contents, inbox.id);
  await markSpaceOpenedAndTrack(context, strip.active.spaceId);
  await transitionActiveSpaceSession(context, () => applyStrip(context, strip, contents, saved));
  recordWorkbenchNavigation(context, strip.active.id);
}

async function loadSpaceContents(spaceIds: readonly string[]) {
  const unique = [...new Set(spaceIds)];
  const contents = await Promise.all(unique.map((spaceId) => loadSpaceSessionProjection(spaceId)));
  return new Map<string, SpaceSessionContent>(unique.map((spaceId, index) => [spaceId, contents[index]!]));
}

function reopenStrip(
  saved: WorkbenchSession | undefined,
  candidates: readonly SavedWorkbenchTab[],
  contents: ReadonlyMap<string, SpaceSessionContent>,
  inboxId: string,
): ReopenedStrip {
  const reopened = new Map<SavedWorkbenchTab, ReopenedTab>();
  for (const candidate of candidates) {
    const entry = reopenTab(candidate, contents.get(candidate.spaceId)!);
    if (entry) reopened.set(candidate, entry);
  }
  const restored = [...reopened.values()];
  const inboxEntry = restored.find(({ tab }) => tab.kind === "space" && tab.spaceId === inboxId) ?? {
    tab: createSpaceCanvasTab(inboxId),
  };
  const entries = restored.includes(inboxEntry) ? restored : [inboxEntry, ...restored];
  const savedActive = saved && saved.activeIndex !== null ? saved.tabs[saved.activeIndex] : undefined;
  return { entries, active: (savedActive && reopened.get(savedActive)?.tab) ?? inboxEntry.tab };
}

function reopenTab(saved: SavedWorkbenchTab, content: SpaceSessionContent): ReopenedTab | undefined {
  if (saved.kind === "space") return { tab: createSpaceCanvasTab(saved.spaceId) };
  const node = content.nodes.find(({ id }) => id === saved.nodeId);
  if (!node || !acceptsEditorContent(saved.kind, node)) return undefined;
  return { tab: createSpaceEditorTab(saved.spaceId, saved.kind, saved.nodeId), node };
}

async function applyStrip(
  context: SpaceSessionContext,
  { entries, active }: ReopenedStrip,
  contents: ReadonlyMap<string, SpaceSessionContent>,
  saved: WorkbenchSession | undefined,
) {
  const tabs = entries.map(({ tab }) => tab);
  for (const spaceId of referencedSpaceIds(tabs)) {
    if (spaceId === active.spaceId) continue;
    const content = contents.get(spaceId)!;
    retainSpaceSession(
      context,
      spaceId,
      content,
      restoreSpaceViewSession(saved?.spaceViews[spaceId], content),
    );
  }
  const activeContent = contents.get(active.spaceId)!;
  activateSpaceSession(
    context,
    active.spaceId,
    activeContent,
    restoreSpaceViewSession(saved?.spaceViews[active.spaceId], activeContent),
  );
  for (const entry of entries) {
    if ("node" in entry) context.editors.open(entry.tab, entry.node);
  }
  if (saved) {
    context.rightSidebarView.value = saved.rightSidebarView;
    context.sourceReturnView.value = saved.rightSidebarView;
  }
  context.tabs.value = tabs;
  selectWorkbenchTab(context, active.id);
  await nextTick();
}

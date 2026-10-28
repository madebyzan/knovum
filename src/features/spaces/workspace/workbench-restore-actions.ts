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
 * Opens the workbench with the tab strip the window last showed. A saved tab returns only while
 * what it shows still exists, so tabs of archived or deleted Spaces and of content that left its
 * Space drop out. The permanent Inbox tab is always present; the saved active tab stays active when
 * it returned, otherwise the Inbox does. Every Space a tab references is loaded, as it is for any
 * open tab: the active one becomes the session and the others are retained behind it.
 */
export async function restoreWorkbench(context: SpaceSessionContext, inbox: Space) {
  const saved = takeWorkbenchSession();
  const available = new Set(
    context.spaces.value.filter(({ archivedAt }) => archivedAt === null).map(({ id }) => id),
  );
  const candidates = (saved?.tabs ?? []).filter(({ spaceId }) => available.has(spaceId));
  const contents = await loadSpaceContents([inbox.id, ...candidates.map(({ spaceId }) => spaceId)]);
  const strip = reopenStrip(saved, candidates, contents, inbox.id);
  await markSpaceOpenedAndTrack(context, strip.active.spaceId);
  await transitionActiveSpaceSession(context, () => applyStrip(context, strip, contents));
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
) {
  const tabs = entries.map(({ tab }) => tab);
  for (const spaceId of referencedSpaceIds(tabs)) {
    if (spaceId !== active.spaceId) retainSpaceSession(context, spaceId, contents.get(spaceId)!);
  }
  activateSpaceSession(context, active.spaceId, contents.get(active.spaceId)!);
  for (const entry of entries) {
    if ("node" in entry) context.editors.open(entry.tab, entry.node);
  }
  context.tabs.value = tabs;
  selectWorkbenchTab(context, active.id);
  await nextTick();
}

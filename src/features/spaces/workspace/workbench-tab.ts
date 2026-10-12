import type { SpaceSurface } from "./space-surface";

interface WorkbenchTabIdentity {
  readonly id: string;
  readonly spaceId: string;
}

/** The canvas of one Space. A Space is loaded while at least one tab references it. */
export interface SpaceCanvasTab extends WorkbenchTabIdentity {
  readonly kind: "space";
}

export interface SpaceEditorTab extends WorkbenchTabIdentity {
  readonly kind: "document" | "video";
  readonly nodeId: string;
}

/** One opened thing in the workbench. The tab strip is the single ordered list of these. */
export type SpaceWorkbenchTab = SpaceCanvasTab | SpaceEditorTab;

export function isSpaceEditorTab(tab: SpaceWorkbenchTab): tab is SpaceEditorTab {
  return tab.kind !== "space";
}

export function workbenchTabSurface(tab: SpaceWorkbenchTab): SpaceSurface {
  return isSpaceEditorTab(tab) ? { kind: tab.kind, nodeId: tab.nodeId } : { kind: "canvas" };
}

export function createSpaceCanvasTab(spaceId: string): SpaceCanvasTab {
  return { id: crypto.randomUUID(), kind: "space", spaceId };
}

export function createSpaceEditorTab(
  spaceId: string,
  kind: SpaceEditorTab["kind"],
  nodeId: string,
): SpaceEditorTab {
  return { id: crypto.randomUUID(), kind, spaceId, nodeId };
}

export function findSpaceCanvasTab(tabs: readonly SpaceWorkbenchTab[], spaceId: string) {
  return tabs.find((tab) => tab.kind === "space" && tab.spaceId === spaceId);
}

export function findEditorTab(tabs: readonly SpaceWorkbenchTab[], target: Omit<SpaceEditorTab, "id">) {
  return tabs.find(
    (tab) =>
      isSpaceEditorTab(tab) &&
      tab.spaceId === target.spaceId &&
      tab.kind === target.kind &&
      tab.nodeId === target.nodeId,
  );
}

/** New tabs open directly right of the active tab; existing order is never rearranged. */
export function tabInsertionIndex(tabs: readonly SpaceWorkbenchTab[], activeTabId: string | undefined) {
  const index = tabs.findIndex(({ id }) => id === activeTabId);
  return index < 0 ? tabs.length : index + 1;
}

/** Closing the active tab moves to its right neighbour, otherwise to its left one. */
export function successorTabId(
  tabs: readonly SpaceWorkbenchTab[],
  closing: ReadonlySet<string>,
  activeTabId: string | undefined,
) {
  if (!activeTabId || !closing.has(activeTabId)) return activeTabId;
  const index = tabs.findIndex(({ id }) => id === activeTabId);
  return (
    tabs.slice(index + 1).find(({ id }) => !closing.has(id))?.id ??
    tabs.slice(0, index).findLast(({ id }) => !closing.has(id))?.id
  );
}

export function referencedSpaceIds(tabs: readonly SpaceWorkbenchTab[]): readonly string[] {
  return [...new Set(tabs.map(({ spaceId }) => spaceId))];
}

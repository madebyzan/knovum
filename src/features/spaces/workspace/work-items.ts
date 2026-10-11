import { computed, shallowRef, type Ref } from "vue";

import type { SpaceSurface } from "./space-surface";

export interface SpaceEditorTab {
  readonly id: string;
  readonly kind: "document" | "video";
  readonly spaceId: string;
  readonly nodeId: string;
}

export function createSpaceWorkItems(activeSpaceId: Ref<string | undefined>) {
  const editorTabs = shallowRef<readonly SpaceEditorTab[]>([]);
  const activeWorkItems = shallowRef<ReadonlyMap<string, string>>(new Map());
  const workspaceDestination = shallowRef<"workbench" | "library">("workbench");
  const surface = computed<SpaceSurface>(() => {
    if (workspaceDestination.value === "library") return { kind: "library" };
    const id = activeSpaceId.value && activeWorkItems.value.get(activeSpaceId.value);
    if (!id) return { kind: "canvas" };
    const tab = editorTabs.value.find((candidate) => candidate.id === id);
    if (!tab || tab.spaceId !== activeSpaceId.value) throw new Error("Active work item is unavailable");
    return { kind: tab.kind, nodeId: tab.nodeId };
  });
  return { editorTabs, activeWorkItems, workspaceDestination, surface };
}

export type SpaceWorkItemState = ReturnType<typeof createSpaceWorkItems>;

export function selectSpaceWorkItem(state: SpaceWorkItemState, spaceId: string, tabId?: string) {
  if (tabId && !state.editorTabs.value.some((tab) => tab.id === tabId && tab.spaceId === spaceId))
    throw new Error("Work item belongs to another Space");
  const items = new Map(state.activeWorkItems.value);
  if (tabId) items.set(spaceId, tabId);
  else items.delete(spaceId);
  state.activeWorkItems.value = items;
  state.workspaceDestination.value = "workbench";
}

export function removeSpaceWorkItems(state: SpaceWorkItemState, ids: ReadonlySet<string>) {
  const remaining = state.editorTabs.value.filter((tab) => !ids.has(tab.id));
  const items = new Map(state.activeWorkItems.value);
  for (const [spaceId, activeId] of items) {
    if (!ids.has(activeId)) continue;
    const siblings = state.editorTabs.value.filter((tab) => tab.spaceId === spaceId);
    const index = siblings.findIndex((tab) => tab.id === activeId);
    const next =
      siblings.slice(index + 1).find((tab) => !ids.has(tab.id)) ??
      siblings.slice(0, index).findLast((tab) => !ids.has(tab.id));
    if (next) items.set(spaceId, next.id);
    else items.delete(spaceId);
  }
  state.activeWorkItems.value = items;
  state.editorTabs.value = remaining;
}

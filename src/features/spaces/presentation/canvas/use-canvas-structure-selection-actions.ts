import type { Ref } from "vue";

import type { useSpaceStore } from "../../session/space-store";
import type { CanvasSelectionProductState } from "./selection-product-state";

interface CanvasStructureSelectionActionsInput {
  store: ReturnType<typeof useSpaceStore>;
  state: Readonly<Ref<CanvasSelectionProductState>>;
  titles: { group: string; section: string };
  run: (operation: () => Promise<unknown>) => void;
}

export function useCanvasStructureSelectionActions({
  store,
  state,
  titles,
  run,
}: CanvasStructureSelectionActionsInput) {
  function createSection() {
    run(() =>
      store.executeSpaceEdit({
        kind: "section.wrap-selection",
        sectionId: crypto.randomUUID(),
        title: titles.section,
        rootIds: state.value.structureNodeIds,
      }),
    );
  }

  function createGroup() {
    run(() =>
      store.executeSpaceEdit({
        kind: "group.create",
        groupId: crypto.randomUUID(),
        rootIds: state.value.structureNodeIds,
        title: titles.group,
      }),
    );
  }

  function dissolveGroup() {
    run(() => store.executeSpaceEdit({ kind: "group.dissolve", rootIds: state.value.structureNodeIds }));
  }

  function fitSection() {
    const section = state.value.section;
    if (section) run(() => store.executeSpaceEdit({ kind: "section.fit", structureNodeId: section.id }));
  }

  function toggleSectionCollapsed() {
    const section = state.value.section;
    if (!section) return;
    run(() =>
      store.executeSpaceEdit({
        kind: "section.set-collapsed",
        structureNodeId: section.id,
        collapsed: !section.collapsed,
      }),
    );
  }

  function dissolveSection() {
    const section = state.value.section;
    if (section) run(() => store.executeSpaceEdit({ kind: "section.dissolve", structureNodeId: section.id }));
  }

  return { createGroup, createSection, dissolveGroup, dissolveSection, fitSection, toggleSectionCollapsed };
}

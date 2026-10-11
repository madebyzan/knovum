import type { Ref } from "vue";

import { i18n } from "../../../../../foundation/i18n/instance";
import { useSpaceStore } from "../../../session/space-store";
import type { CanvasSelectionProductState } from "./selection-product-state";

interface CanvasStructureSelectionActionsInput {
  state: Readonly<Ref<CanvasSelectionProductState>>;
  run: (operation: () => Promise<unknown>) => void;
}

export function createCanvasStructureSelectionActions({ state, run }: CanvasStructureSelectionActionsInput) {
  const store = useSpaceStore();

  function createSection() {
    run(() =>
      store.executeSpaceEdit({
        kind: "section.wrap-selection",
        sectionId: crypto.randomUUID(),
        title: i18n.global.t("spaces.canvas.sections.untitled"),
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
        title: i18n.global.t("spaces.canvas.groups.untitled"),
      }),
    );
  }

  function dissolveGroup() {
    run(() => store.executeSpaceEdit({ kind: "group.dissolve", rootIds: state.value.structureNodeIds }));
  }

  function fitSection() {
    const section = state.value.section;
    if (section && state.value.canFitSection)
      run(() => store.executeSpaceEdit({ kind: "section.fit", structureNodeId: section.id }));
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
    if (section && state.value.canDissolveSection)
      run(() => store.executeSpaceEdit({ kind: "section.dissolve", structureNodeId: section.id }));
  }

  return { createGroup, createSection, dissolveGroup, dissolveSection, fitSection, toggleSectionCollapsed };
}

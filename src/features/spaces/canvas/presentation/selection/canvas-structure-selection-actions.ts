import type { Ref } from "vue";

import { i18n } from "../../../../../foundation/i18n/instance";
import type { ContentFillPreset } from "../../../model/structure/schema/content-appearance";
import { useSpaceStore } from "../../../session/space-store";
import { canvasAppearanceUpdate } from "../../schema/document";
import type { SectionSize } from "./section-frame-size";
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

  /** The selected Section, when the selection lets it be edited. */
  function editableSection() {
    return state.value.canEditSection ? state.value.section : undefined;
  }

  function dissolveSection() {
    const section = editableSection();
    if (section) run(() => store.executeSpaceEdit({ kind: "section.dissolve", structureNodeId: section.id }));
  }

  function setSectionAutoLayout(autoLayout: boolean) {
    const section = editableSection();
    if (section && section.autoLayout !== autoLayout)
      run(() =>
        store.executeSpaceEdit({ kind: "section.update-layout", structureNodeId: section.id, autoLayout }),
      );
  }

  /** A preset fill, or none to show the canvas panel behind the Section. */
  function setSectionFill(fill: ContentFillPreset | undefined) {
    const section = editableSection();
    if (!section || section.appearance.fill === fill) return;
    const appearance = { ...section.appearance, customFill: undefined, fill };
    run(() =>
      store.executeSpaceEdit({
        kind: "section.update-layout",
        structureNodeId: section.id,
        appearance: canvasAppearanceUpdate(appearance),
      }),
    );
  }

  function resizeSection(size: SectionSize) {
    const section = editableSection();
    if (!section) return;
    const { frame } = section;
    if (frame.width === size.width && frame.height === size.height) return;
    run(() =>
      store.executeSpaceEdit({
        kind: "section.resize-frame",
        structureNodeId: section.id,
        frame: { x: frame.x, y: frame.y, ...size },
      }),
    );
  }

  return {
    createGroup,
    createSection,
    dissolveGroup,
    dissolveSection,
    fitSection,
    resizeSection,
    setSectionAutoLayout,
    setSectionFill,
    toggleSectionCollapsed,
  };
}

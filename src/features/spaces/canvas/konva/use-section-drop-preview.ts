import { useTimeoutFn } from "@vueuse/core";
import { onScopeDispose, shallowRef, type Ref } from "vue";

import type { SpaceEditSnapshot } from "../../model/structure/schema/edit-snapshot";
import {
  sectionDropPreview,
  sectionDropIndex,
  type SectionDropPreview,
} from "../../structure/section-drop-reparent";

interface SectionDropPreviewOptions {
  snapshot(): SpaceEditSnapshot;
  temporarilyExpandedSectionId: Ref<string | undefined>;
}

export function useSectionDropPreview(options: SectionDropPreviewOptions) {
  const state = shallowRef<SectionDropPreview>();
  let pendingExpansionId: string | undefined;
  const expansion = useTimeoutFn(
    (candidate: string) => {
      pendingExpansionId = undefined;
      options.temporarilyExpandedSectionId.value = candidate;
    },
    600,
    { immediate: false },
  );

  function clear() {
    expansion.stop();
    pendingExpansionId = undefined;
    state.value = undefined;
    options.temporarilyExpandedSectionId.value = undefined;
  }

  function update(rootIds: readonly string[], delta: { x: number; y: number } | undefined) {
    if (!delta) return clear();
    const snapshot = options.snapshot();
    const expandedId = options.temporarilyExpandedSectionId.value;
    const preview = sectionDropPreview(
      { snapshot, rootIds, delta },
      expandedId ? { allowCollapsedSectionId: expandedId } : {},
    );
    if (!preview) return clear();
    const candidate = preview.sectionId;
    const layout = sectionDropIndex(snapshot).layouts.get(candidate);
    const collapsed = layout?.kind === "section" && layout.collapsed;
    if (!collapsed || preview.kind === "leave") {
      clear();
      state.value = preview;
      return;
    }
    state.value = preview;
    if (expandedId === candidate) {
      return;
    }
    if (pendingExpansionId === candidate) return;
    clear();
    state.value = preview;
    pendingExpansionId = candidate;
    expansion.start(candidate);
  }

  onScopeDispose(clear);
  return { state, clear, update };
}

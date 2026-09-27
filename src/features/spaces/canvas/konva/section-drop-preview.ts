import { useTimeoutFn } from "@vueuse/core";
import { onBeforeUnmount, type Ref, type ShallowRef } from "vue";

import { sectionDropCandidateId } from "../../organization/section-drop-reparent";
import type { SpaceEditSnapshot } from "../contract/space-edit-snapshot";

export function useSectionDropPreview(options: {
  dropTargetId: ShallowRef<string | undefined>;
  snapshot(): SpaceEditSnapshot;
  temporarilyExpandedSectionId: Ref<string | undefined>;
}) {
  let pendingExpansionId: string | undefined;
  const expansion = useTimeoutFn(
    (candidate: string) => {
      pendingExpansionId = undefined;
      options.temporarilyExpandedSectionId.value = candidate;
      options.dropTargetId.value = candidate;
    },
    600,
    { immediate: false },
  );

  function clear() {
    expansion.stop();
    pendingExpansionId = undefined;
    options.dropTargetId.value = undefined;
    options.temporarilyExpandedSectionId.value = undefined;
  }

  function update(rootIds: readonly string[], delta: { x: number; y: number } | undefined) {
    if (!delta) return clear();
    const snapshot = options.snapshot();
    const candidate = sectionDropCandidateId(snapshot, rootIds, delta, true);
    const collapsed =
      candidate &&
      snapshot.layouts.some(
        (layout) => layout.structureNodeId === candidate && layout.kind === "section" && layout.collapsed,
      );
    if (!candidate || !collapsed) {
      clear();
      options.dropTargetId.value = candidate;
      return;
    }
    if (options.temporarilyExpandedSectionId.value === candidate) {
      options.dropTargetId.value = candidate;
      return;
    }
    options.dropTargetId.value = undefined;
    if (pendingExpansionId === candidate) return;
    clear();
    pendingExpansionId = candidate;
    expansion.start(candidate);
  }

  onBeforeUnmount(clear);
  return { clear, update };
}

import { computed, type Ref } from "vue";

import { sectionDrawCandidateIds } from "../../organization/section-draw-candidates";
import type { CanvasRect } from "../contract/canvas-editor-port";
import type { SpaceEditSnapshot } from "../contract/space-edit-snapshot";

export function useSectionDrawCandidateBounds(
  draft: Readonly<Ref<CanvasRect | undefined>>,
  snapshot: () => SpaceEditSnapshot,
  structureBounds: Readonly<Ref<ReadonlyMap<string, CanvasRect>>>,
) {
  return computed(() => {
    const frame = draft.value;
    if (!frame) return [];
    return sectionDrawCandidateIds(snapshot(), frame).flatMap((id) => {
      const bounds = structureBounds.value.get(id);
      return bounds ? [bounds] : [];
    });
  });
}

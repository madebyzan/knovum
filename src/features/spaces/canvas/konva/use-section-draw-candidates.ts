import { compact } from "es-toolkit";
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
    return compact(sectionDrawCandidateIds(snapshot(), frame).map((id) => structureBounds.value.get(id)));
  });
}

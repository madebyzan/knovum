import { compact } from "es-toolkit";
import { computed, type Ref } from "vue";

import type { SpaceEditSnapshot } from "../../model/structure/schema/edit-snapshot";
import { sectionDrawCandidateIds } from "../../structure/section-draw-candidates";
import type { CanvasRect } from "../contracts/canvas-editor-port";

export function useSectionDrawCandidateBounds(
  draft: Readonly<Ref<CanvasRect | undefined>>,
  snapshot: () => SpaceEditSnapshot,
  structureBoundsById: Readonly<Ref<ReadonlyMap<string, CanvasRect>>>,
) {
  return computed(() => {
    const frame = draft.value;
    if (!frame) return [];
    return compact(sectionDrawCandidateIds(snapshot(), frame).map((id) => structureBoundsById.value.get(id)));
  });
}

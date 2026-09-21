import { computed, type Ref } from "vue";

import { createStructureIndex } from "../../model/structure/structure-index";
import { structureParentTargets } from "../../organization/structure-capabilities";
import type { useSpaceStore } from "../../session/space-store";
import type { CanvasSelectionProductState } from "./selection-product-state";

export function useCanvasReparentActions(
  store: ReturnType<typeof useSpaceStore>,
  state: Readonly<Ref<CanvasSelectionProductState>>,
  spaceRootTitle: string,
  run: (operation: () => Promise<unknown>) => void,
) {
  const targets = computed(() => {
    return structureParentTargets(store.editSnapshot, state.value.structureNodeIds, spaceRootTitle);
  });

  function reparent(parentId: string | null) {
    const index = createStructureIndex(store.editSnapshot.structure);
    run(() =>
      store.executeSpaceEdit({
        kind: "structure.reparent",
        rootIds: state.value.structureNodeIds,
        parentId,
        order: (index.childrenByParent.get(parentId) ?? []).length,
      }),
    );
  }

  return { reparent, targets };
}

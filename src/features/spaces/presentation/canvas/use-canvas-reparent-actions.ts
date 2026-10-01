import { computed, type Ref } from "vue";

import { createStructureIndex } from "../../model/structure/structure-index";
import type { useSpaceStore } from "../../session/space-store";
import { structureParentTargets } from "../../structure/structure-capabilities";
import type { CanvasSelectionProductState } from "./selection-product-state";

interface CanvasReparentActionsInput {
  store: ReturnType<typeof useSpaceStore>;
  state: Readonly<Ref<CanvasSelectionProductState>>;
  spaceRootTitle: string;
  run: (operation: () => Promise<unknown>) => void;
}

export function useCanvasReparentActions({ store, state, spaceRootTitle, run }: CanvasReparentActionsInput) {
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

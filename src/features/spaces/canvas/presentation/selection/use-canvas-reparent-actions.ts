import { computed, type Ref } from "vue";
import { useI18n } from "vue-i18n";

import { createStructureIndex } from "../../../model/structure/structure-index";
import { useSpaceStore } from "../../../session/space-store";
import { structureParentTargets } from "../../../structure/action-availability";
import type { CanvasSelectionProductState } from "./selection-product-state";

interface CanvasReparentActionsInput {
  state: Readonly<Ref<CanvasSelectionProductState>>;
  run: (operation: () => Promise<unknown>) => void;
}

export function useCanvasReparentActions({ state, run }: CanvasReparentActionsInput) {
  const store = useSpaceStore();
  const { t } = useI18n();
  const targets = computed(() => {
    return structureParentTargets(
      store.editSnapshot,
      state.value.structureNodeIds,
      t("spaces.canvas.spaceRoot"),
    );
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

import { shallowRef, type Ref } from "vue";

import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { useSpaceStore } from "../../session/space-store";
import type { CreationDialogId } from "./creation-dialog";
import { useDerivationActions } from "./use-derivation-actions";
import { useDerivationCatalog } from "./use-derivation-catalog";

interface SelectionDerivationsOptions {
  store: ReturnType<typeof useSpaceStore>;
  selection: Ref<SpaceCanvasNode[]>;
}

export function useSelectionDerivations({ store, selection }: SelectionDerivationsOptions) {
  const dialog = shallowRef<CreationDialogId>();
  const catalogState = useDerivationCatalog(selection);
  const actions = useDerivationActions({
    store,
    selection,
    single: catalogState.single,
    sources: catalogState.sources,
    subtitleInputs: catalogState.subtitleInputs,
    dialog,
  });

  return {
    single: catalogState.single,
    decisions: catalogState.decisions,
    sourceTitle: catalogState.sourceTitle,
    sources: catalogState.sources,
    dialog,
    subtitleInputs: catalogState.subtitleInputs,
    ...actions,
  };
}

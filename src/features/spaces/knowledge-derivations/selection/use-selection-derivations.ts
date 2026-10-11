import { computed, shallowRef, type Ref } from "vue";

import { canRunAction } from "../../../../shared/contracts/action-availability";
import { useNodeOperationGate } from "../../canvas/use-node-operation-gate";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { useSpaceStore } from "../../session/space-store";
import {
  CREATION_INTENT_DEFINITIONS,
  type CreationIntentDecisions,
} from "../catalog/creation-intent-definitions";
import type { CreationDialogId } from "./creation-dialog";
import { createDerivationActions } from "./derivation-actions";
import { useDerivationCatalog } from "./use-derivation-catalog";

export function useSelectionDerivations(selection: Ref<SpaceCanvasNode[]>) {
  const store = useSpaceStore();
  const dialog = shallowRef<CreationDialogId>();
  const catalogState = useDerivationCatalog(selection);
  const gate = useNodeOperationGate();
  const decisions = computed<CreationIntentDecisions>(
    () =>
      Object.fromEntries(
        CREATION_INTENT_DEFINITIONS.map((definition) => {
          const decision = catalogState.decisions.value[definition.id];
          return [
            definition.id,
            canRunAction(decision) && !gate.allows("derive")
              ? { state: "blocked", reason: "active-node-operation" }
              : decision,
          ];
        }),
      ) as CreationIntentDecisions,
  );
  const actions = createDerivationActions({
    store,
    selection,
    single: catalogState.single,
    sources: catalogState.sources,
    dialog,
    canRun: (actionId) => canRunAction(decisions.value[actionId]),
    canSubmit: () => gate.allows("derive"),
  });

  return {
    single: catalogState.single,
    decisions,
    sourceTitle: catalogState.sourceTitle,
    sources: catalogState.sources,
    dialog,
    ...actions,
  };
}

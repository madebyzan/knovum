import { computed, shallowRef, type Ref } from "vue";

import { canRunAction } from "../../../../shared/contracts/action-availability";
import { useNodeOperationGate } from "../../canvas/use-node-operation-gate";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { creationIntentCatalog } from "../catalog/creation-intent-catalog";
import {
  CREATION_INTENT_DEFINITIONS,
  type CreationIntentDecisions,
} from "../catalog/creation-intent-definitions";
import type { CreationOutput } from "../creation";
import { createDerivationActions } from "./derivation-actions";

export function useSelectionDerivations(selection: Ref<SpaceCanvasNode[]>) {
  const composing = shallowRef<CreationOutput>();
  const catalog = computed(() => creationIntentCatalog(selection.value));
  const single = computed(() => catalog.value.single);
  const sources = computed(() => catalog.value.sources);
  const gate = useNodeOperationGate();
  const decisions = computed<CreationIntentDecisions>(
    () =>
      Object.fromEntries(
        CREATION_INTENT_DEFINITIONS.map((definition) => {
          const decision = catalog.value.decisions[definition.id];
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
    single,
    sources,
    composing,
    canRun: (actionId) => canRunAction(decisions.value[actionId]),
    canSubmit: () => gate.allows("derive"),
  });

  return { single, decisions, sources, composing, ...actions };
}

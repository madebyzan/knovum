import { useQuery } from "@tanstack/vue-query";
import { computed, type Ref } from "vue";

import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceImagePayload } from "../../model/nodes/node-payload";
import { isDerivationComparable, prepareDerivationComparison } from "./comparison-commands";

/**
 * Whether a derived picture lays over the input it was made from, and — once the comparison is
 * opened — that input. Both answers are the backend's alone: it knows the transform that made the
 * output and whether the file it read still exists. The node bar mounts on selection, so each
 * selection asks again.
 */
export function useDerivationComparison(
  node: Readonly<Ref<SpaceCanvasNode<SpaceImagePayload>>>,
  open: Readonly<Ref<boolean>>,
) {
  const key = computed(() => [
    "spaces",
    node.value.id,
    "derivation-comparison",
    node.value.data.sourceRevision,
  ]);
  const comparable = useQuery(
    computed(() => ({
      queryKey: key.value,
      queryFn: () => isDerivationComparable(node.value.id),
      enabled: Boolean(node.value.data.sourceRevision),
    })),
  );
  const input = useQuery(
    computed(() => ({
      queryKey: [...key.value, "input"],
      queryFn: () => prepareDerivationComparison(node.value.id),
      enabled: open.value && comparable.data.value === true,
      retry: false,
    })),
  );
  return {
    available: computed(() => comparable.data.value === true),
    input: computed(() => input.data.value),
    error: computed(() => input.error.value),
  };
}

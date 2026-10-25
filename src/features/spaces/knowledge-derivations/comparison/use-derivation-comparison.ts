import { useQuery } from "@tanstack/vue-query";
import { computed, type Ref } from "vue";

import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceImagePayload } from "../../model/nodes/node-payload";
import { isLineageRelation } from "../../model/relations/lineage";
import { useSpaceStore } from "../../session/space-store";
import { isDerivationComparable, prepareDerivationComparison } from "./comparison-commands";

/**
 * Whether a derived picture can be laid over what it was made from, and — once someone opens the
 * comparison — the input to lay it over. Both answers are the backend's: it alone knows which
 * transform made the output and whether the file it read still exists. The key carries what can
 * change them: the output's asset and the asset of the source its lineage points to.
 */
export function useDerivationComparison(
  node: Readonly<Ref<SpaceCanvasNode<SpaceImagePayload>>>,
  open: Readonly<Ref<boolean>>,
) {
  const store = useSpaceStore();
  const sourceRevision = computed(() => {
    const lineage = store.relations.find(
      (relation) => isLineageRelation(relation) && relation.sourceId === node.value.id,
    );
    const source = lineage && store.nodes.find(({ id }) => id === lineage.targetId);
    return source?.data.kind === "image" ? source.data.sourceRevision : undefined;
  });
  const key = computed(
    () =>
      [
        "spaces",
        node.value.id,
        "derivation-comparison",
        node.value.data.sourceRevision,
        sourceRevision.value,
      ] as const,
  );
  const comparable = useQuery(
    computed(() => ({
      queryKey: key.value,
      queryFn: () => isDerivationComparable(node.value.id),
      enabled: Boolean(node.value.data.sourceRevision && sourceRevision.value),
    })),
  );
  const input = useQuery(
    computed(() => ({
      queryKey: [...key.value, "input"] as const,
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

import { computed, type Ref } from "vue";

import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import { creationIntentCatalog } from "../catalog/creation-intent-catalog";

export function useDerivationCatalog(selection: Ref<SpaceCanvasNode[]>) {
  const catalog = computed(() => creationIntentCatalog(selection.value));
  const single = computed(() => catalog.value.single);
  const sources = computed(() => catalog.value.sources);
  const decisions = computed(() => catalog.value.decisions);
  const sourceTitle = computed(() => selection.value[0]?.data.title ?? "");

  return {
    catalog,
    single,
    sources,
    decisions,
    sourceTitle,
  };
}

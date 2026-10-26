import { computed, type MaybeRefOrGetter, toValue } from "vue";

import { spaceNodePreviewSource } from "../../../assets/node-preview-source";
import { nodeIcon } from "../../../canvas/presentation/node-icon";
import { useNodesAtWork } from "../../../canvas/presentation/nodes/use-node-work-task";
import { useSpaceStore } from "../../../session/space-store";
import type { CreationSource } from "../../catalog/creation-source";
import type { CanvasGenerator } from "../canvas-generator";
import { pinnedReferences, readReference, type ReferenceReading } from "../generator-references";

/** One reference as the composer shows it: what it is, how it looks, and whether it can be read. */
export type ReferenceView = ReferenceReading & {
  nodeId: string;
  title: string;
  icon: ReturnType<typeof nodeIcon>;
  preview: string | undefined;
};

/** The generator's references, read against the canvas as it is now. */
export function useReferenceReadings(generator: MaybeRefOrGetter<CanvasGenerator>) {
  const space = useSpaceStore();
  const atWork = useNodesAtWork();
  const nodes = computed(() => new Map(space.nodes.map((node) => [node.id, node])));

  const references = computed<ReferenceView[]>(() =>
    toValue(generator).references.flatMap((nodeId) => {
      const node = nodes.value.get(nodeId);
      if (!node) return [];
      return [
        {
          ...readReference(node, atWork.value.has(nodeId)),
          nodeId,
          title: node.data.title,
          icon: nodeIcon(node.data.kind),
          preview: spaceNodePreviewSource(node.data),
        },
      ];
    }),
  );
  /** What a Run would be given now; undefined while any reference cannot be read yet. */
  const sources = computed<CreationSource[] | undefined>(() => pinnedReferences(references.value));
  const preparing = computed(() => references.value.some(({ state }) => state === "preparing"));
  const unreadable = computed(() => references.value.some(({ state }) => state === "unreadable"));

  return { references, sources, preparing, unreadable };
}

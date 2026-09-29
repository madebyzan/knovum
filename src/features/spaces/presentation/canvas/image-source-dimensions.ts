import { defineStore } from "pinia";
import { shallowRef, watch } from "vue";

import { imageEditOf, rotatedImageSize } from "../../image-editing/image-edit";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceImagePayload } from "../../model/nodes/node-payload";
import { useSpaceStore } from "../../session/space-store";

export function imageRemovalSize(
  node: SpaceCanvasNode<SpaceImagePayload>,
  dimensions: ReturnType<typeof useImageSourceDimensions>,
) {
  const loaded = dimensions.forNode(node.id, node.data.sourceRevision);
  const width = node.data.intrinsicWidth ?? loaded?.width;
  const height = node.data.intrinsicHeight ?? loaded?.height;
  if (!width || !height) return undefined;
  const { crop, rotation } = imageEditOf(node.data);
  return rotatedImageSize(width * crop.width, height * crop.height, rotation);
}

export function imageRemovalSizeValid(size: { width: number; height: number } | undefined) {
  return Boolean(
    size && [size.width, size.height].every((side) => Math.ceil(side) >= 512 && Math.ceil(side) <= 4096),
  );
}

export const useImageSourceDimensions = defineStore("spaces-image-source-dimensions", () => {
  const space = useSpaceStore();
  const dimensions = shallowRef(new Map<string, { revision: string; width: number; height: number }>());

  function set(nodeId: string, revision: string, width: number, height: number) {
    dimensions.value = new Map(dimensions.value).set(nodeId, { revision, width, height });
  }

  function clear(nodeId: string) {
    const next = new Map(dimensions.value);
    next.delete(nodeId);
    dimensions.value = next;
  }

  watch(
    () => space.activeSpaceId,
    () => (dimensions.value = new Map()),
    { flush: "sync" },
  );
  watch(
    () => space.nodes,
    (nodes) => {
      const ids = new Set(nodes.map(({ id }) => id));
      dimensions.value = new Map([...dimensions.value].filter(([id]) => ids.has(id)));
    },
    { flush: "sync" },
  );

  return {
    set,
    clear,
    forNode: (nodeId: string, revision?: string) => {
      const current = dimensions.value.get(nodeId);
      return current?.revision === revision ? current : undefined;
    },
  };
});

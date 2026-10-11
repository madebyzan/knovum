import { defineStore } from "pinia";
import { shallowRef, watch } from "vue";

import { imageEditOf, rotatedImageSize } from "../../image-editing/image-edit";
import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceImagePayload } from "../../model/nodes/node-payload";
import { useSpaceStore } from "../../session/space-store";

export function imageRemovalSize(
  node: SpaceCanvasNode<SpaceImagePayload>,
  loaded: { width: number; height: number } | undefined,
) {
  const width = node.data.intrinsicWidth ?? loaded?.width;
  const height = node.data.intrinsicHeight ?? loaded?.height;
  if (!width || !height) return undefined;
  const { crop, rotation } = imageEditOf(node.data);
  return rotatedImageSize(width * crop.width, height * crop.height, rotation);
}

export function imageRemovalSizeValid(size: { width: number; height: number } | undefined) {
  return Boolean(
    size &&
    size.width * size.height <= 16_777_216 &&
    [size.width, size.height].every((side) => Math.ceil(side) >= 1 && Math.ceil(side) <= 16384),
  );
}

export const useImageSourceDimensionsStore = defineStore("spaces-image-source-dimensions", () => {
  const space = useSpaceStore();
  const dimensionsByNodeId = shallowRef(
    new Map<string, { revision: string; width: number; height: number }>(),
  );

  function set(nodeId: string, revision: string, width: number, height: number) {
    dimensionsByNodeId.value = new Map(dimensionsByNodeId.value).set(nodeId, { revision, width, height });
  }

  function clear(nodeId: string) {
    const next = new Map(dimensionsByNodeId.value);
    next.delete(nodeId);
    dimensionsByNodeId.value = next;
  }

  watch(
    () => space.activeSpaceId,
    () => (dimensionsByNodeId.value = new Map()),
    { flush: "sync" },
  );
  watch(
    () => space.nodes,
    (nodes) => {
      const ids = new Set(nodes.map(({ id }) => id));
      dimensionsByNodeId.value = new Map([...dimensionsByNodeId.value].filter(([id]) => ids.has(id)));
    },
    { flush: "sync" },
  );

  return {
    set,
    clear,
    forNode: (nodeId: string, revision?: string) => {
      const current = dimensionsByNodeId.value.get(nodeId);
      return current?.revision === revision ? current : undefined;
    },
  };
});

import { computed, type Ref } from "vue";

import type { SpaceCanvasNode } from "../../model/nodes/canvas-node";
import type { SpaceImagePayload } from "../../model/nodes/node-payload";

export function useCanvasSelectedMedia(selectedNodes: Readonly<Ref<SpaceCanvasNode[]>>) {
  const single = computed(() => (selectedNodes.value.length === 1 ? selectedNodes.value[0] : undefined));
  const selectedImageNode = computed(() => {
    const node = single.value;
    return node && isImageNode(node) ? node : undefined;
  });
  const selectedVideoNode = computed(() => {
    const node = single.value;
    return node?.data.kind === "video" ? node : undefined;
  });
  return { selectedImageNode, selectedVideoNode };
}

function isImageNode(node: SpaceCanvasNode): node is SpaceCanvasNode<SpaceImagePayload> {
  return node.data.kind === "image";
}

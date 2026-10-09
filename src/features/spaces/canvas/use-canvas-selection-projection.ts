import { computed, toValue, type MaybeRefOrGetter } from "vue";

import type { SpaceCanvasNode } from "../model/nodes/canvas-node";
import { getCanvasEditorSession, selectedCanvasNodesInOrder } from "./editor-session";

export function useCanvasSelectionProjection(nodes: MaybeRefOrGetter<readonly SpaceCanvasNode[]>) {
  const session = getCanvasEditorSession();
  const selectedNodes = computed(() =>
    selectedCanvasNodesInOrder(toValue(nodes), session.selection.value.nodeIds),
  );
  return { selection: session.selection, selectedNodes };
}

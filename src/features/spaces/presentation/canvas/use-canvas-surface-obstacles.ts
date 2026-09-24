import { computed, shallowRef, watch } from "vue";

import type { CanvasDocumentV1 } from "../../canvas/contract/canvas-document";
import type { CanvasRect } from "../../canvas/contract/canvas-editor-port";
import { useCanvasEditorSession } from "../../canvas/editor-session";
import { canvasFrameBounds } from "../../canvas/konva/interaction/canvas-snap-geometry";
import { useSpaceStore } from "../../session/space-store";
import type { SurfaceRect } from "./selection-surface-placement";

/** Every selection surface shares the visible canvas geometry, regardless of node type. */
export function useCanvasSurfaceObstacles() {
  const session = useCanvasEditorSession();
  const store = useSpaceStore();
  const document = shallowRef<CanvasDocumentV1>();

  watch(
    session.activeEditor,
    (editor, _previous, onCleanup) => {
      document.value = editor?.getDocument();
      if (editor) onCleanup(editor.observeDocument((next) => (document.value = next)));
    },
    { immediate: true },
  );

  const hidden = computed(() => new Set(store.nodes.filter((node) => node.hidden).map((node) => node.id)));

  function near(selection: CanvasRect, reach: number): SurfaceRect[] {
    const selected = new Set([
      ...session.selection.value.nodeIds,
      ...session.selection.value.structureNodeIds,
    ]);
    const obstacles: SurfaceRect[] = [];
    for (const { nodeId, frame } of document.value?.nodes ?? []) {
      if (selected.has(nodeId) || hidden.value.has(nodeId)) continue;
      const bounds = canvasFrameBounds(frame);
      if (
        bounds.right < selection.x - reach ||
        bounds.left > selection.x + selection.width + reach ||
        bounds.bottom < selection.y - reach ||
        bounds.top > selection.y + selection.height + reach
      )
        continue;
      obstacles.push(bounds);
    }
    return obstacles;
  }

  return { near };
}

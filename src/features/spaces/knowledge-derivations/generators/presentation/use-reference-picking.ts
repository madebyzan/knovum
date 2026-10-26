import { useEventListener } from "@vueuse/core";
import { shallowRef, watch } from "vue";

import { getCanvasEditorSession } from "../../../canvas/editor-session";
import { useNodesAtWork } from "../../../canvas/presentation/nodes/use-node-work-task";
import { useCanvasSelectionProjection } from "../../../canvas/use-canvas-selection-projection";
import { useSpaceStore } from "../../../session/space-store";
import { readReference } from "../generator-references";
import { useCanvasGeneratorStore } from "../generator-store";

/**
 * Taking references from the canvas: while the active generator is picking, what the person clicks
 * on the canvas (one node, several with Shift, or a marquee) becomes a reference — the canvas's
 * own selection is the gesture, so nothing about clicking has to be learnt again. Content a model
 * could never read is refused by name instead of being added and failing later. Esc finishes.
 */
export function useReferencePicking() {
  const store = useCanvasGeneratorStore();
  const space = useSpaceStore();
  const editorSession = getCanvasEditorSession();
  const atWork = useNodesAtWork();
  const { selectedNodes } = useCanvasSelectionProjection(() => space.nodes);
  /** The title of what was just refused, until the next pick. */
  const refused = shallowRef<string>();

  watch(
    () => store.picking,
    () => {
      refused.value = undefined;
      // Picks are gestures, not a selection to act on: picking starts and ends with nothing selected.
      editorSession.selectNodes([]);
    },
  );

  watch(selectedNodes, (nodes) => {
    const generator = store.active;
    if (!store.picking || !generator || nodes.length === 0) return;
    const readable = nodes.filter(
      (node) => readReference(node, atWork.value.has(node.id)).state !== "unreadable",
    );
    store.addReferences(
      generator.id,
      readable.map(({ id }) => id),
    );
    refused.value = nodes.find((node) => !readable.includes(node))?.data.title;
  });

  useEventListener(
    globalThis,
    "keydown",
    (event: KeyboardEvent) => {
      if (!store.picking || event.key !== "Escape") return;
      event.preventDefault();
      event.stopPropagation();
      store.stopPicking();
    },
    { capture: true },
  );

  return { refused };
}

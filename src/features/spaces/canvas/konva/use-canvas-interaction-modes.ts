import { computed } from "vue";

import type { CanvasSurfaceProps } from "../contracts/canvas-surface";
import type { useKonvaCanvasRuntime } from "./use-konva-canvas-runtime";

/**
 * Which pointer interactions the surface offers right now, and the cursor that says so. Selecting
 * needs the select tool with edits allowed and no pan in progress; holding Space or an active
 * gesture suspends hit-testing; a drag hides the selection chrome it would otherwise draw.
 */
export function useCanvasInteractionModes(
  props: CanvasSurfaceProps,
  runtime: ReturnType<typeof useKonvaCanvasRuntime>,
) {
  const { viewport } = runtime;
  const selectGesturesEnabled = computed(
    () =>
      props.interactionEnabled &&
      props.canCommitCanvasGestures &&
      props.tool === "select" &&
      !viewport.panning.value,
  );
  const interactive = computed(
    () =>
      selectGesturesEnabled.value &&
      !runtime.gestures.busy.value &&
      !runtime.capturesCanvasInteraction.value &&
      !viewport.spacePressed.value,
  );
  const selectionChromeVisible = computed(
    () =>
      selectGesturesEnabled.value &&
      !viewport.spacePressed.value &&
      runtime.directManipulation.value !== "drag",
  );
  /** Node content takes the pointer whenever the select tool is idle, edits allowed or not. */
  const contentInteractionEnabled = computed(
    () =>
      props.interactionEnabled &&
      props.tool === "select" &&
      !viewport.spacePressed.value &&
      !viewport.panning.value,
  );
  const cursorClass = computed(() => {
    if (runtime.connectors.draft.value) return "cursor-crosshair";
    if (viewport.panning.value) return "cursor-grabbing";
    if (viewport.spacePressed.value || props.tool === "pan") return "cursor-grab";
    return ["note", "text", "section"].includes(props.tool) ? "cursor-crosshair" : "";
  });
  return { interactive, selectionChromeVisible, contentInteractionEnabled, cursorClass };
}

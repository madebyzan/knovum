import { watch, type Reactive, type Ref } from "vue";

import type { CanvasCamera } from "./canvas-camera";

/**
 * Keeps world content anchored to the screen when the host moves rather than only resizes.
 *
 * A workbench panel animation does not resize the window, it moves one edge of the work area. When that edge
 * is the leading one, the stage origin travels with it, so every node slides across the screen even though
 * the camera never changed - a sidebar appears to drag the canvas along with it, while the opposite edge
 * leaves content alone. Folding the host's origin shift into the camera makes both edges behave the same: the
 * viewport narrows, the content stays where the user left it.
 *
 * The host's own geometry observation drives this, because an origin shift within the window is always caused
 * by a layout change that resizes the host too. The shift is projected in the same frame as the layout that
 * caused it; a camera write that waited for the next frame would be visible as jitter against that layout.
 */
export function useCameraHostAnchor(
  camera: CanvasCamera,
  host: Readonly<Ref<HTMLElement | null>>,
  size: Reactive<{ height: number; width: number }>,
) {
  let origin: { x: number; y: number } | undefined;

  function anchor() {
    const element = host.value;
    if (!element) {
      origin = undefined;
      return;
    }
    const { left, top } = element.getBoundingClientRect();
    const previous = origin;
    origin = { x: left, y: top };
    if (!previous || (previous.x === left && previous.y === top)) return;
    const current = camera.get();
    camera.set({ x: current.x - (left - previous.x), y: current.y - (top - previous.y), zoom: current.zoom });
    camera.flush();
  }

  watch([host, () => size.width, () => size.height], anchor, { flush: "post", immediate: true });
}

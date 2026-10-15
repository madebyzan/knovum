import { onScopeDispose, watch, type Reactive, type Ref } from "vue";

import type { CanvasCamera } from "./canvas-camera";

/**
 * Keeps world content anchored to the screen when the host moves rather than only resizes.
 *
 * A panel changes the work area's width in one step, so this runs once per toggle, not once per frame. The
 * trailing edge leaves content alone - the stage origin stays put and the canvas is only clipped differently
 * - while the leading edge drags the origin along, which would slide every node across the screen although
 * the camera never changed. Folding the host's origin shift into the camera makes both edges behave the
 * same: the viewport narrows and content stays where the user left it.
 *
 * The shift is projected in the same frame as the layout that caused it; a camera write that waited for the
 * next frame would be visible as a one-frame jump against that layout.
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
  onScopeDispose(() => (origin = undefined));
}

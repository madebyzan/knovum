import { useDebounceFn } from "@vueuse/core";
import { onScopeDispose, watch, type Reactive, type Ref } from "vue";

import type { CanvasCamera } from "./canvas-camera";

/** Quiet time after the last geometry change before the compensation is folded into the camera. */
const SETTLE_MS = 160;

/**
 * Keeps world content anchored to the screen when the host moves rather than only resizes.
 *
 * A workbench panel animation does not resize the window, it moves one edge of the work area. The trailing
 * edge leaves content alone: the stage origin stays put, so the canvas is only clipped differently and the
 * compositor does all of it. The leading edge drags the stage origin along, so content would slide across
 * the screen although the camera never changed.
 *
 * Compensating by writing the camera on every frame of that animation would be geometrically right and
 * visibly worse: each write redraws every layer, reprojects the overlay and the grid, so the leading edge
 * would cost a full repaint per frame where the trailing edge costs nothing. While the host shrinks the
 * compensation is therefore a transform on the content layer - one composited property, no repaint, which is
 * exactly what makes the trailing edge smooth - and only once the geometry stops moving is the accumulated
 * shift folded into the camera and the transform cleared, in one frame, as a single redraw.
 *
 * A growing host is not symmetric: it uncovers area that has never been painted, and no transform can fill
 * it. There the camera adopts the shift in the same frame, which is the one case where the repaint is the
 * work being asked for rather than waste.
 *
 * The transform lives on the content layer rather than the host, because the host clips the canvas; moving
 * it would carry the clip along and let the canvas escape its panel. Pointer coordinates stay correct
 * throughout, since the vendor derives them from the content element's own client rect.
 */
export function useCameraHostAnchor(
  camera: CanvasCamera,
  host: Readonly<Ref<HTMLElement | null>>,
  content: Readonly<Ref<HTMLElement | null>>,
  size: Reactive<{ height: number; width: number }>,
) {
  let layout: { x: number; y: number } | undefined;
  let offset = { x: 0, y: 0 };

  function applyOffset() {
    const node = content.value;
    if (!node) return;
    const moved = offset.x !== 0 || offset.y !== 0;
    node.style.transform = moved ? `translate3d(${-offset.x}px, ${-offset.y}px, 0)` : "";
  }

  function adopt() {
    if (offset.x === 0 && offset.y === 0) return;
    const current = camera.get();
    camera.set({ x: current.x - offset.x, y: current.y - offset.y, zoom: current.zoom });
    offset = { x: 0, y: 0 };
    applyOffset();
    camera.flush();
  }

  const scheduleAdopt = useDebounceFn(adopt, SETTLE_MS);

  function anchor() {
    const element = host.value;
    if (!element) {
      layout = undefined;
      return;
    }
    // The host's own rect is unaffected by the compensation, which lives on its child.
    const { left, top } = element.getBoundingClientRect();
    const previous = layout;
    layout = { x: left, y: top };
    if (!previous || (previous.x === left && previous.y === top)) return;
    offset = { x: offset.x + (left - previous.x), y: offset.y + (top - previous.y) };
    applyOffset();
    // Shrinking exposes nothing, so the compositor can hold the shift until the geometry settles. Growing
    // exposes area that no transform can fill - those pixels have never been drawn - so the camera adopts
    // the shift in the same frame and the newly uncovered content is painted with it.
    if (left < previous.x || top < previous.y) adopt();
    else void scheduleAdopt();
  }

  watch([host, content, () => size.width, () => size.height], anchor, { flush: "post", immediate: true });
  onScopeDispose(adopt);
}

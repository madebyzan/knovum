import { onScopeDispose, watch, type Ref } from "vue";

import type { CanvasViewport } from "../../contracts/canvas-editor-port";
import type { CanvasCamera } from "./canvas-camera";

/**
 * The declarations that put a world-space layer under the camera, and publish its zoom so content can
 * stay screen-sized inside it.
 */
export function worldTransform({ x, y, zoom }: CanvasViewport): Record<string, string> {
  return { transform: `translate(${x}px, ${y}px) scale(${zoom})`, "--canvas-zoom": String(zoom) };
}

/**
 * Keeps one element aligned with the live camera.
 *
 * Elements that are anchored to world coordinates but sized in screen pixels cannot wait for the settled
 * camera: a reactive style binding would re-render their component on every frame of a pan. They write the
 * camera-dependent declarations imperatively instead, which keeps the component tree idle while the camera
 * moves and leaves the remaining style declarations to normal template bindings. Declarations are CSS
 * property names, custom properties included, so descendants can derive their own camera-dependent styles.
 */
export function useCameraStyle(
  camera: CanvasCamera,
  element: Readonly<Ref<HTMLElement | null>>,
  declarations: (camera: CanvasViewport) => Record<string, string>,
) {
  function apply(value: CanvasViewport) {
    const target = element.value;
    if (!target) return;
    for (const [property, declaration] of Object.entries(declarations(value)))
      target.style.setProperty(property, declaration);
  }

  const stop = camera.observeFrames(apply);
  watch(element, () => apply(camera.get()), { flush: "post", immediate: true });
  onScopeDispose(stop);
}

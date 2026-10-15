import { onScopeDispose, watch, type Ref } from "vue";

import type { CanvasViewport } from "../../contracts/canvas-editor-port";
import type { CanvasCamera } from "./canvas-camera";

/**
 * Keeps one element aligned with the live camera.
 *
 * Elements that are anchored to world coordinates but sized in screen pixels cannot wait for the settled
 * camera: a reactive style binding would re-render their component on every frame of a pan. They write the
 * camera-dependent declarations imperatively instead, which keeps the component tree idle while the camera
 * moves and leaves the remaining style declarations to normal template bindings.
 */
export function useCameraStyle(
  camera: CanvasCamera,
  element: Readonly<Ref<HTMLElement | null>>,
  declarations: (camera: CanvasViewport) => Record<string, string>,
) {
  function apply(value: CanvasViewport) {
    const target = element.value;
    if (target) Object.assign(target.style, declarations(value));
  }

  const stop = camera.observeFrames(apply);
  watch(element, () => apply(camera.get()), { flush: "post", immediate: true });
  onScopeDispose(stop);
}

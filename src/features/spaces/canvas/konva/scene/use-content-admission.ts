import { onScopeDispose, shallowRef, watch, type Ref } from "vue";

import type { CanvasCamera } from "../viewport/canvas-camera";
import { admitVisibleIds, CONTENT_MOUNT_BUDGET } from "./visibility-admission";

/**
 * Publishes the node content that may mount, draining the culling target under a frame budget.
 *
 * Geometry follows culling immediately; content does not. While the camera moves, the frame already pays
 * for the pan, so revealed cards mount a few per frame and the lead overscan covers the rest. Once the
 * camera settles the frame is free again and the remaining target is admitted at once, which is also why
 * opening a space or editing the scene mounts its content without a stagger.
 */
export function useContentAdmission(camera: CanvasCamera, target: Readonly<Ref<ReadonlySet<string>>>) {
  const admitted = shallowRef<ReadonlySet<string>>(target.value);
  let moving = false;
  let frame: number | undefined;

  function admit(budget: number) {
    const next = admitVisibleIds(admitted.value, target.value, budget);
    if (next) admitted.value = next.ids;
    return next?.pending ?? false;
  }

  function drain() {
    frame = undefined;
    if (admit(CONTENT_MOUNT_BUDGET)) schedule();
  }

  function schedule() {
    if (frame === undefined) frame = requestAnimationFrame(drain);
  }

  function cancel() {
    if (frame === undefined) return;
    cancelAnimationFrame(frame);
    frame = undefined;
  }

  watch(target, () => (moving ? schedule() : void admit(target.value.size)));
  const stopFrames = camera.observeFrames(() => {
    moving = true;
    schedule();
  });
  const stopSettled = camera.observeSettled(() => {
    moving = false;
    cancel();
    admit(target.value.size);
  });
  onScopeDispose(() => {
    stopFrames();
    stopSettled();
    cancel();
  });
  return admitted;
}

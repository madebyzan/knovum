import { onBeforeUnmount, shallowRef } from "vue";

import type { CanvasGestureUpdate } from "./canvas-gesture-update";

export function useLayoutPreview() {
  const gestureUpdateByNodeId = shallowRef(new Map<string, CanvasGestureUpdate>());
  let request: number | undefined;
  let queued: readonly CanvasGestureUpdate[] = [];

  function update(next: readonly CanvasGestureUpdate[]) {
    if (!next.length) {
      if (request !== undefined) cancelAnimationFrame(request);
      request = undefined;
      queued = [];
      gestureUpdateByNodeId.value = new Map();
      return;
    }
    queued = next;
    if (request !== undefined) return;
    request = requestAnimationFrame(() => {
      request = undefined;
      gestureUpdateByNodeId.value = new Map(queued.map((layout) => [layout.nodeId, layout]));
    });
  }

  onBeforeUnmount(() => {
    if (request !== undefined) cancelAnimationFrame(request);
  });
  return { gestureUpdateByNodeId, update };
}

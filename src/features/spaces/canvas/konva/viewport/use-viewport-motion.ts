import { onBeforeUnmount, shallowRef } from "vue";

export function useViewportMotion() {
  const moving = shallowRef(false);
  let settleFrame: number | undefined;

  function markMoving() {
    moving.value = true;
    if (settleFrame !== undefined) cancelAnimationFrame(settleFrame);
    settleFrame = requestAnimationFrame(() => {
      settleFrame = requestAnimationFrame(() => {
        settleFrame = undefined;
        moving.value = false;
      });
    });
  }

  onBeforeUnmount(() => {
    if (settleFrame !== undefined) cancelAnimationFrame(settleFrame);
  });

  return { moving, markMoving };
}

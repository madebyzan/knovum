import { useEventListener, useTimeoutFn } from "@vueuse/core";
import { shallowRef } from "vue";

/** Matches the panel geometry transition in `app.css`, plus a frame of slack. */
const MOTION_MS = 220;

/**
 * Tracks the workbench panel geometry transition.
 *
 * While a panel animates its own size, the workspace must stay free of work that cannot finish inside a
 * frame. Panel content that is still being revealed therefore mounts after the motion instead of during it,
 * and any layout change that invalidates the animation - a window resize, a splitter drag - cancels it.
 */
export function usePanelMotion() {
  const active = shallowRef(false);
  const expandingRightSidebar = shallowRef(false);
  const { start, stop } = useTimeoutFn(settle, MOTION_MS, { immediate: false });

  function settle() {
    active.value = false;
    expandingRightSidebar.value = false;
  }

  function begin() {
    active.value = true;
    start();
  }

  function cancel() {
    stop();
    settle();
  }

  useEventListener(globalThis, "resize", cancel);
  return {
    active,
    begin,
    beginRightSidebarExpansion() {
      expandingRightSidebar.value = true;
      begin();
    },
    cancel,
    expandingRightSidebar,
  };
}

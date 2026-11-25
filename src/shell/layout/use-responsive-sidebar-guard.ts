import { useEventListener } from "@vueuse/core";
import { watch, type Ref } from "vue";

const MIN_MAIN_WORKSPACE_WIDTH = 480;

export interface ResponsiveSidebarGuardOptions {
  sidebarPinned: Ref<boolean>;
  sidebarWidth: Readonly<Ref<number>>;
  rightSidebarOpen: Readonly<Ref<boolean>>;
  rightSidebarWidth: Readonly<Ref<number>>;
  collapseSidebarEphemerally: () => void;
}

/**
 * Keeps at least 480px for the primary workspace surface when both sidebars are open on narrow
 * desktop windows. Collapsing is ephemeral so the user's persisted sidebar preference is preserved.
 */
export function useResponsiveSidebarGuard(options: ResponsiveSidebarGuardOptions) {
  function enforceMainWorkspaceBudget() {
    if (!options.sidebarPinned.value || !options.rightSidebarOpen.value) return;
    const viewportWidth = globalThis.innerWidth;
    if (!Number.isFinite(viewportWidth) || viewportWidth <= 0) return;
    const remaining = viewportWidth - options.sidebarWidth.value - options.rightSidebarWidth.value;
    if (remaining < MIN_MAIN_WORKSPACE_WIDTH) {
      options.collapseSidebarEphemerally();
    }
  }

  useEventListener(globalThis, "resize", enforceMainWorkspaceBudget);
  watch(
    [options.sidebarPinned, options.rightSidebarOpen, options.sidebarWidth, options.rightSidebarWidth],
    enforceMainWorkspaceBudget,
    { immediate: true },
  );
}

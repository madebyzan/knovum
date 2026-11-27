import { useEventListener } from "@vueuse/core";
import { computed, shallowRef, watch, type Ref } from "vue";

const MIN_MAIN_WORKSPACE_WIDTH = 480;

export interface ResponsiveSidebarGuardOptions {
  sidebarPinned: Readonly<Ref<boolean>>;
  sidebarWidth: Readonly<Ref<number>>;
  rightSidebarOpen: Readonly<Ref<boolean>>;
  rightSidebarWidth: Readonly<Ref<number>>;
  closeSidebarPreview: () => void;
  closeRightSidebar: () => void;
}

/**
 * Keeps at least 480px for the primary workspace surface when both sidebars are open on narrow
 * desktop windows without mutating the persisted left-sidebar preference. Widening the window or
 * closing the right sidebar restores the left sidebar automatically; explicitly reopening the left
 * sidebar while narrow closes the right sidebar (last-opened wins).
 */
export function useResponsiveSidebarGuard(options: ResponsiveSidebarGuardOptions) {
  const sidebarAutoCollapsed = shallowRef(false);
  const effectiveSidebarPinned = computed(() => options.sidebarPinned.value && !sidebarAutoCollapsed.value);

  function isBelowMainWorkspaceBudget(): boolean {
    const viewportWidth = globalThis.innerWidth;
    if (!Number.isFinite(viewportWidth) || viewportWidth <= 0) return false;
    return (
      viewportWidth - options.sidebarWidth.value - options.rightSidebarWidth.value < MIN_MAIN_WORKSPACE_WIDTH
    );
  }

  function evaluateBudget() {
    if (!options.sidebarPinned.value || !options.rightSidebarOpen.value) {
      sidebarAutoCollapsed.value = false;
      return;
    }
    if (isBelowMainWorkspaceBudget()) {
      if (!sidebarAutoCollapsed.value) {
        sidebarAutoCollapsed.value = true;
        options.closeSidebarPreview();
      }
      return;
    }
    sidebarAutoCollapsed.value = false;
  }

  function reclaimLeftSidebar(): boolean {
    if (!sidebarAutoCollapsed.value) return false;
    sidebarAutoCollapsed.value = false;
    if (options.rightSidebarOpen.value && isBelowMainWorkspaceBudget()) {
      options.closeRightSidebar();
    }
    return true;
  }

  useEventListener(globalThis, "resize", evaluateBudget);
  watch([options.sidebarPinned, options.rightSidebarOpen], evaluateBudget, { immediate: true });

  return {
    effectiveSidebarPinned,
    sidebarAutoCollapsed,
    evaluateBudget,
    reclaimLeftSidebar,
  };
}

import { afterEach, describe, expect, it, vi } from "vitest";
import { effectScope, nextTick, ref } from "vue";

import { useResponsiveSidebarGuard } from "../use-responsive-sidebar-guard";

const originalInnerWidth = globalThis.innerWidth;

function setViewportWidth(width: number) {
  Object.defineProperty(globalThis, "innerWidth", { value: width, configurable: true });
}

afterEach(() => {
  setViewportWidth(originalInnerWidth);
});

describe("responsive sidebar guard", () => {
  it("keeps both sidebars open when at least 480px remain for the main workspace", async () => {
    setViewportWidth(1280);
    const sidebarPinned = ref(true);
    const rightSidebarOpen = ref(true);
    const closePreview = vi.fn<() => void>();
    const closeRight = vi.fn<() => void>();
    const scope = effectScope();
    const guard = scope.run(() =>
      useResponsiveSidebarGuard({
        sidebarPinned,
        sidebarWidth: ref(240),
        rightSidebarOpen,
        rightSidebarWidth: ref(360),
        closeSidebarPreview: closePreview,
        closeRightSidebar: closeRight,
      }),
    )!;
    await nextTick();
    expect(closePreview).not.toHaveBeenCalled();
    expect(guard.effectiveSidebarPinned.value).toBe(true);
    expect(sidebarPinned.value).toBe(true);
    scope.stop();
  });

  it("auto-collapses left sidebar without mutating preference and restores when space returns", async () => {
    setViewportWidth(1280);
    const sidebarPinned = ref(true);
    const rightSidebarOpen = ref(false);
    const closePreview = vi.fn<() => void>();
    const closeRight = vi.fn<() => void>(() => {
      rightSidebarOpen.value = false;
    });
    const scope = effectScope();
    const guard = scope.run(() =>
      useResponsiveSidebarGuard({
        sidebarPinned,
        sidebarWidth: ref(240),
        rightSidebarOpen,
        rightSidebarWidth: ref(360),
        closeSidebarPreview: closePreview,
        closeRightSidebar: closeRight,
      }),
    )!;

    setViewportWidth(960);
    globalThis.dispatchEvent(new Event("resize"));
    await nextTick();
    expect(guard.effectiveSidebarPinned.value).toBe(true);

    rightSidebarOpen.value = true;
    await nextTick();
    expect(closePreview).toHaveBeenCalledTimes(1);
    expect(guard.effectiveSidebarPinned.value).toBe(false);
    expect(sidebarPinned.value).toBe(true);

    // Widening restores the left sidebar automatically.
    setViewportWidth(1280);
    globalThis.dispatchEvent(new Event("resize"));
    await nextTick();
    expect(guard.effectiveSidebarPinned.value).toBe(true);

    // Reclaiming while narrow closes the right sidebar (last-opened wins).
    setViewportWidth(960);
    globalThis.dispatchEvent(new Event("resize"));
    await nextTick();
    expect(guard.effectiveSidebarPinned.value).toBe(false);
    expect(guard.reclaimLeftSidebar()).toBe(true);
    await nextTick();
    expect(closeRight).toHaveBeenCalledTimes(1);
    expect(guard.effectiveSidebarPinned.value).toBe(true);
    scope.stop();
  });
});

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
    const collapse = vi.fn<() => void>(() => {
      sidebarPinned.value = false;
    });
    const scope = effectScope();
    scope.run(() => {
      useResponsiveSidebarGuard({
        sidebarPinned,
        sidebarWidth: ref(240),
        rightSidebarOpen,
        rightSidebarWidth: ref(360),
        collapseSidebarEphemerally: collapse,
      });
    });
    await nextTick();
    expect(collapse).not.toHaveBeenCalled();
    expect(sidebarPinned.value).toBe(true);
    scope.stop();
  });

  it("collapses the left sidebar ephemerally when both sidebars squeeze the workspace below 480px", async () => {
    setViewportWidth(1280);
    const sidebarPinned = ref(true);
    const rightSidebarOpen = ref(false);
    const collapse = vi.fn<() => void>(() => {
      sidebarPinned.value = false;
    });
    const scope = effectScope();
    scope.run(() => {
      useResponsiveSidebarGuard({
        sidebarPinned,
        sidebarWidth: ref(240),
        rightSidebarOpen,
        rightSidebarWidth: ref(360),
        collapseSidebarEphemerally: collapse,
      });
    });

    setViewportWidth(960);
    globalThis.dispatchEvent(new Event("resize"));
    await nextTick();
    expect(collapse).not.toHaveBeenCalled();

    rightSidebarOpen.value = true;
    await nextTick();
    expect(collapse).toHaveBeenCalledTimes(1);
    expect(sidebarPinned.value).toBe(false);
    scope.stop();
  });
});

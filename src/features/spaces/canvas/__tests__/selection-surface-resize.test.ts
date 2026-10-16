import { mount } from "@vue/test-utils";
import { describe, expect, it, vi } from "vitest";
import { computed, nextTick, shallowRef, type Ref } from "vue";

const context = vi.hoisted(() => ({ height: null as null | Ref<number> }));
vi.mock("@vueuse/core", () => ({
  useElementSize: (element: Ref<HTMLElement | null>) => ({
    width: computed(() => (element.value?.classList.contains("inset-0") ? 900 : 240)),
    height: computed(() => {
      if (element.value?.classList.contains("inset-0")) return 700;
      const limit = Number.parseFloat(element.value?.style.maxHeight ?? "");
      return Math.min(context.height!.value, Number.isFinite(limit) ? limit : Infinity);
    }),
  }),
}));
vi.mock("../editor-session", () => ({
  getCanvasEditorSession: () => ({ viewport: shallowRef({}) }),
}));
vi.mock("../presentation/use-surface-trigger-open", () => ({
  useSurfaceTriggerOpen: () => shallowRef(false),
}));
import SpaceCanvasSelectionAnchor from "../presentation/selection/SpaceCanvasSelectionAnchor.vue";

describe("selection surface intrinsic measurement", () => {
  it("repositions a growing editor instead of retaining the previous mode's height limit", async () => {
    context.height = shallowRef(120);
    const wrapper = mount(SpaceCanvasSelectionAnchor, {
      props: {
        bounds: { x: 220, y: 360, width: 160, height: 200 },
        canvasToViewportPoint: (point) => point,
        safeArea: { top: 8, right: 16, bottom: 8, left: 16 },
        reserved: [],
        selectionKey: "image",
        gap: 12,
        kind: "compact-editor",
      },
      slots: { default: "<section>Editor content and actions</section>" },
    });
    await nextTick();
    const surface = wrapper.get<HTMLElement>("[data-side]");
    expect(surface.element.style.maxHeight).toBe("120px");
    context.height.value = 220;
    await nextTick();
    expect(surface.element.style.maxHeight).toBe("220px");
    expect(surface.element.style.transform).toContain(", 128px, 0)");
    expect(surface.element.style.left).toBe("");
    expect(surface.element.style.top).toBe("");
    context.height.value = 900;
    await nextTick();
    expect(surface.attributes("data-side")).toBe("right");
    expect(surface.element.style.maxHeight).toBe("684px");
    context.height.value = 120;
    await nextTick();
    expect(surface.element.style.maxHeight).toBe("120px");
    expect(surface.attributes("data-side")).toBe("above");
    wrapper.unmount();
  });
});

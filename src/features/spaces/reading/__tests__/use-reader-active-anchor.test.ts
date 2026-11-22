import { flushPromises, mount } from "@vue/test-utils";
import { describe, expect, it, onTestFinished, vi } from "vitest";
import { defineComponent, h, ref } from "vue";

import { useReaderActiveAnchor } from "../presentation/use-reader-active-anchor";

function createSection(index: string, top: number) {
  const el = document.createElement("article");
  el.dataset.sectionIndex = index;
  el.getBoundingClientRect = () => ({
    top,
    bottom: top + 120,
    left: 0,
    right: 400,
    width: 400,
    height: 120,
    x: 0,
    y: top,
    toJSON: () => ({}),
  });
  return el;
}

describe("useReaderActiveAnchor", () => {
  it("skips unindexed intro sections, pins clicked targets during smooth scroll, and resumes on manual wheel", async () => {
    const host = document.createElement("div");
    let scrollTop = 0;
    let scrollHeight = 1200;
    const scrollToSpy = vi.fn<(opts: ScrollToOptions) => void>((opts) => {
      scrollTop = opts.top ?? scrollTop;
    });
    Object.defineProperties(host, {
      scrollTop: {
        get: () => scrollTop,
        set: (v: number) => {
          scrollTop = v;
        },
      },
      scrollHeight: { get: () => scrollHeight },
      clientHeight: { get: () => 500 },
    });
    host.scrollTo = scrollToSpy as unknown as typeof host.scrollTo;
    host.getBoundingClientRect = () => ({
      top: 100,
      bottom: 600,
      left: 0,
      right: 400,
      width: 400,
      height: 500,
      x: 0,
      y: 100,
      toJSON: () => ({}),
    });

    const intro = createSection("0", 120);
    const sec1 = createSection("1", 320);
    const sec2 = createSection("2", 620);
    const sec3 = createSection("3", 920);
    host.append(intro, sec1, sec2, sec3);

    const scroller = ref<HTMLElement | null>(host);
    let api!: ReturnType<typeof useReaderActiveAnchor>;
    const Host = defineComponent({
      setup() {
        api = useReaderActiveAnchor({
          scroller,
          selector: "[data-section-index]",
          keyOf: (el) => el.dataset.sectionIndex,
          validIds: () => ["1", "2", "3"],
          sourceKey: () => "doc-1",
        });
        return () => h("div");
      },
    });
    const wrapper = mount(Host);
    onTestFinished(() => wrapper.unmount());
    await flushPromises();

    // Intro ("0") is not in validIds, so the first indexed section ("1") is active at top.
    expect(api.activeId.value).toBe("1");

    // Clicking "3" pins "3" immediately and scrolls container-relative to 24px below host top.
    api.scrollTo("3", sec3);
    expect(api.activeId.value).toBe("3");
    expect(scrollToSpy).toHaveBeenCalledWith({ top: 796, behavior: "smooth" });

    // Programmatic scroll events while pinned do not revert the clicked item.
    host.dispatchEvent(new Event("scroll"));
    expect(api.activeId.value).toBe("3");

    // Manual wheel scrolling back up releases the pin and tracks the topmost section crossing the activation line.
    scrollTop = 300;
    sec1.getBoundingClientRect = () => ({
      top: 40,
      bottom: 160,
      left: 0,
      right: 400,
      width: 400,
      height: 120,
      x: 0,
      y: 40,
      toJSON: () => ({}),
    });
    sec2.getBoundingClientRect = () => ({
      top: 140,
      bottom: 260,
      left: 0,
      right: 400,
      width: 400,
      height: 120,
      x: 0,
      y: 140,
      toJSON: () => ({}),
    });
    sec3.getBoundingClientRect = () => ({
      top: 440,
      bottom: 560,
      left: 0,
      right: 400,
      width: 400,
      height: 120,
      x: 0,
      y: 440,
      toJSON: () => ({}),
    });
    host.dispatchEvent(new Event("wheel"));
    api.sync();
    expect(api.activeId.value).toBe("2");

    // Scrolling to the bottom activates the last valid item.
    scrollTop = 696;
    scrollHeight = 1200;
    api.sync();
    expect(api.activeId.value).toBe("3");
  });
});

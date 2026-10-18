import { mount } from "@vue/test-utils";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { nextTick } from "vue";

import { i18n } from "../../../../foundation/i18n/instance";
import type { DocumentResourceProjection } from "../presentation/documents/document-resource-projection";
import SpaceDocumentResourceGrid from "../presentation/documents/SpaceDocumentResourceGrid.vue";

vi.mock("@tauri-apps/api/core", () => ({
  convertFileSrc: (path: string) => `asset://${path}`,
}));
let viewportWidth = 320;
const observers = new Set<TestResizeObserver>();

class TestResizeObserver implements ResizeObserver {
  private targets = new Set<Element>();
  constructor(private callback: ResizeObserverCallback) {
    observers.add(this);
  }
  observe(target: Element) {
    this.targets.add(target);
  }
  unobserve(target: Element) {
    this.targets.delete(target);
  }
  disconnect() {
    this.targets.clear();
    observers.delete(this);
  }
  resize() {
    const entries = [...this.targets].map((target) => {
      const size = {
        inlineSize: (target as HTMLElement).offsetWidth,
        blockSize: (target as HTMLElement).offsetHeight,
      };
      return {
        target,
        borderBoxSize: [size],
        contentBoxSize: [size],
        devicePixelContentBoxSize: [size],
        contentRect: target.getBoundingClientRect(),
      };
    });
    this.callback(entries, this);
  }
}

const resource: DocumentResourceProjection = {
  resource: {
    id: "a".repeat(64),
    path: "/assets/document-a.png",
    mediaType: "image/png",
    width: 1200,
    height: 800,
    byteSize: 4096,
  },
  occurrences: [],
  locationLabel: "PDF · p.2",
  sourceKind: "rendered",
};

describe("document resource grid", () => {
  let wrapper: ReturnType<typeof mount>;
  beforeEach(() => {
    viewportWidth = 320;
    vi.stubGlobal("ResizeObserver", TestResizeObserver);
    vi.spyOn(HTMLElement.prototype, "offsetWidth", "get").mockImplementation(() => viewportWidth);
    vi.spyOn(HTMLElement.prototype, "offsetHeight", "get").mockImplementation(function (this: HTMLElement) {
      if (this.hasAttribute("data-index")) {
        const preview = this.querySelector<HTMLElement>("[data-resource-preview]");
        return Math.round(Number.parseFloat(preview?.style.height ?? "0"));
      }
      return 384;
    });
  });
  afterEach(() => {
    wrapper?.unmount();
    observers.clear();
    vi.unstubAllGlobals();
  });
  async function render(resources: readonly DocumentResourceProjection[]) {
    wrapper = mount(SpaceDocumentResourceGrid, {
      props: { resources, sourceRevision: "revision-1" },
      global: { plugins: [i18n] },
      attachTo: document.body,
    });
    await nextTick();
    return wrapper;
  }

  it("renders a resource preview and emits the typed selection", async () => {
    await render([resource]);

    expect(wrapper.get("img").attributes("src")).toContain("document-a.png");
    expect(wrapper.text()).toContain("PDF · p.2");
    await wrapper.get("button").trigger("click");
    expect(wrapper.emitted("select")).toEqual([[resource]]);
  });

  it("packs mixed image ratios into the shortest lane and adapts to sidebar resizing", async () => {
    await render([resource, item(1, 800, 1200), item(2, 1600, 400), item(3, 1200, 800)]);
    const tiles = () => wrapper.findAll<HTMLElement>("[data-index]");
    expect(tiles()[0]!.element.style.width).toBe("156px");
    expect(tiles()[1]!.element.style.transform).toBe("translate(164px, 0px)");
    expect(tiles()[2]!.element.style.transform).toBe("translate(0px, 112px)");
    const previews = wrapper.findAll<HTMLElement>("[data-resource-preview]");
    expect(previews[0]!.element.style.height).toBe("104px");
    expect(previews[1]!.element.style.height).toBe("234px");
    expect(previews[2]!.element.style.height).toBe("64px");

    viewportWidth = 420;
    for (const observer of observers) observer.resize();
    await nextTick();
    expect(tiles()[0]!.element.style.width).toBe("206px");
    expect(tiles()[1]!.element.style.transform).toBe("translate(214px, 0px)");
    expect(wrapper.get("img").attributes("width")).toBe("1200");
  });

  it("virtualizes large documents while keeping the focused image and its next tab target mounted", async () => {
    await render(Array.from({ length: 1000 }, (_, index) => item(index, 1200, 800)));
    expect(wrapper.findAll("button").length).toBeLessThan(30);
    const last = wrapper.findAll<HTMLElement>("[data-index]").at(-1)!;
    const index = Number(last.attributes("data-index"));
    last.get<HTMLButtonElement>("button").element.focus();
    await nextTick();
    expect(wrapper.find(`[data-index="${index + 1}"]`).exists()).toBe(true);
    const scroller = wrapper.get<HTMLElement>('[role="list"]');
    scroller.element.scrollTop = 1000;
    await scroller.trigger("scroll");
    expect(wrapper.find(`[data-index="${index}"]`).exists()).toBe(true);
    expect(document.activeElement).toBe(last.get("button").element);
  });

  it("shows image failure in place and clears it for a new source revision", async () => {
    await render([resource]);
    await wrapper.get("img").trigger("error");
    expect(wrapper.text()).toContain(i18n.global.t("spaces.processing.imageUnavailable"));
    expect(wrapper.find("img").exists()).toBe(false);
    await wrapper.setProps({ sourceRevision: "revision-2" });
    expect(wrapper.get("img").attributes("src")).toContain("revision-2");
  });
});

function item(index: number, width: number, height: number): DocumentResourceProjection {
  return {
    ...resource,
    resource: { ...resource.resource, id: index.toString(16).padStart(64, "0"), width, height },
  };
}

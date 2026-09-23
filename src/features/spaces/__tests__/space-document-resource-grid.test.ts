import { shallowMount } from "@vue/test-utils";
import { describe, expect, it, vi } from "vitest";

import { i18n } from "../../../foundation/i18n/i18n";
import type { DocumentResourceProjection } from "../presentation/document/document-resource-projection";
import SpaceDocumentResourceGrid from "../presentation/document/SpaceDocumentResourceGrid.vue";

vi.mock("@tauri-apps/api/core", () => ({
  convertFileSrc: (path: string) => `asset://${path}`,
}));
vi.mock("@tanstack/vue-virtual", async () => {
  const { computed } = await import("vue");
  return {
    useVirtualizer: (options: { value: { count: number; estimateSize: () => number } }) =>
      computed(() => ({
        getTotalSize: () => options.value.count * options.value.estimateSize(),
        measureElement: () => undefined,
        getVirtualItems: () =>
          Array.from({ length: options.value.count }, (_, index) => ({
            index,
            key: index,
            size: options.value.estimateSize(),
            start: index * options.value.estimateSize(),
          })),
      })),
  };
});

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
  it("renders a resource preview and emits the typed selection", async () => {
    const wrapper = shallowMount(SpaceDocumentResourceGrid, {
      props: { resources: [resource], sourceRevision: "revision-1" },
      global: { plugins: [i18n] },
    });

    expect(wrapper.get("img").attributes("src")).toContain("document-a.png");
    expect(wrapper.text()).toContain("PDF · p.2");
    await wrapper.get("button").trigger("click");
    expect(wrapper.emitted("select")).toEqual([[resource]]);
  });
});

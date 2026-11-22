import { enableAutoUnmount, mount } from "@vue/test-utils";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { defineComponent, ref } from "vue";

import { i18n } from "../../../../../foundation/i18n/instance";
import type { GroundedCitation } from "../../../knowledge/model/schema/grounded-citation";
import SpaceCitationPreview from "../SpaceCitationPreview.vue";
import {
  CITATION_HOVER_CLOSE_DELAY_MS,
  CITATION_HOVER_OPEN_DELAY_MS,
  useCitationHover,
} from "../use-citation-hover";

enableAutoUnmount(afterEach);

const SAMPLE_CITATIONS: GroundedCitation[] = [
  {
    index: 2,
    state: "current",
    nodeId: "video-1",
    chunkId: "chunk-2",
    nodeTitle: "Emma Watson Interview",
    sourceRevision: "rev-1",
    locator: { sourceRevision: "rev-1", location: { kind: "media", startMs: 92_000, endMs: 96_000 } },
    quote: "I auditioned when I was nine and started filming when I was ten.",
  },
];

function mountHarness(onOpen: (citation: GroundedCitation) => void) {
  const Harness = defineComponent({
    components: { SpaceCitationPreview },
    setup() {
      const citations = ref(SAMPLE_CITATIONS);
      const hover = useCitationHover(citations);
      return { hover, onOpen };
    },
    template: `
      <div
        data-body
        @pointerover="hover.onBodyPointerOver"
        @pointerout="hover.onBodyPointerOut"
      >
        <button type="button" class="ui-citation" data-citation-index="2">2</button>
        <SpaceCitationPreview
          :state="hover.active.value"
          @open="onOpen"
          @preview-enter="hover.onPreviewPointerEnter"
          @preview-leave="hover.onPreviewPointerLeave"
        />
      </div>
    `,
  });
  return mount(Harness, { attachTo: document.body, global: { plugins: [i18n] } });
}

describe("Inline citation hover preview", () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
    document.body.innerHTML = "";
  });

  it("delays opening on hover so quick pointer sweeps never flash previews", async () => {
    const onOpen = vi.fn<(citation: GroundedCitation) => void>();
    const wrapper = mountHarness(onOpen);
    const trigger = wrapper.get("[data-citation-index='2']");

    await trigger.trigger("pointerover");
    vi.advanceTimersByTime(CITATION_HOVER_OPEN_DELAY_MS - 50);
    await wrapper.vm.$nextTick();
    expect(document.body.querySelector("[data-citation-preview]")).toBeNull();

    await trigger.trigger("pointerout");
    vi.advanceTimersByTime(CITATION_HOVER_OPEN_DELAY_MS + 100);
    await wrapper.vm.$nextTick();
    expect(document.body.querySelector("[data-citation-preview]")).toBeNull();
  });

  it("shows citation details after the hover delay, bridges onto the preview, and opens on click", async () => {
    const onOpen = vi.fn<(citation: GroundedCitation) => void>();
    const wrapper = mountHarness(onOpen);
    const trigger = wrapper.get("[data-citation-index='2']");

    await trigger.trigger("pointerover");
    vi.advanceTimersByTime(CITATION_HOVER_OPEN_DELAY_MS);
    await wrapper.vm.$nextTick();

    const preview = document.body.querySelector<HTMLElement>("[data-citation-preview]");
    expect(preview).not.toBeNull();
    expect(preview?.textContent).toContain("Emma Watson Interview");
    expect(preview?.textContent).toContain("1:32");
    expect(preview?.textContent).toContain("I auditioned when I was nine");

    await trigger.trigger("pointerout");
    vi.advanceTimersByTime(CITATION_HOVER_CLOSE_DELAY_MS - 50);
    preview!.dispatchEvent(new Event("pointerenter"));
    vi.advanceTimersByTime(CITATION_HOVER_CLOSE_DELAY_MS + 100);
    await wrapper.vm.$nextTick();
    expect(document.body.querySelector("[data-citation-preview]")).not.toBeNull();

    preview!.click();
    expect(onOpen).toHaveBeenCalledWith(SAMPLE_CITATIONS[0]);
  });
});

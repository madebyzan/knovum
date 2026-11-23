import { mount } from "@vue/test-utils";
import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { nextTick } from "vue";

import { i18n } from "../../../../../foundation/i18n/instance";
import { useSpaceStore } from "../../../session/space-store";
import SpaceCanvasBackgroundControl from "../SpaceCanvasBackgroundControl.vue";

vi.mock("../../../enrichment/use-enrichment-job-observation", () => ({
  useEnrichmentJobObservation: () => {},
}));

describe("SpaceCanvasBackgroundControl", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
  });

  it("orbits the pattern choices above the icon and applies the picked one", async () => {
    const store = useSpaceStore();
    const wrapper = mount(SpaceCanvasBackgroundControl, {
      attachTo: document.body,
      global: { plugins: [i18n] },
    });

    const trigger = wrapper.get("button[aria-expanded]");
    expect(trigger.attributes("aria-expanded")).toBe("false");
    expect(document.body.querySelectorAll("[data-pattern-orbit]")).toHaveLength(3);

    await trigger.trigger("click");
    expect(trigger.attributes("aria-expanded")).toBe("true");

    const grid = document.body.querySelector<HTMLElement>('[data-pattern-value="grid"]');
    expect(grid?.getAttribute("aria-pressed")).toBe("false");
    grid?.click();
    await nextTick();
    expect(store.backgroundPattern).toBe("grid");
    expect(trigger.attributes("aria-expanded")).toBe("false");

    await trigger.trigger("click");
    const relitGrid = document.body.querySelector<HTMLElement>('[data-pattern-value="grid"]');
    expect(relitGrid?.getAttribute("aria-pressed")).toBe("true");

    document.body.querySelector("[data-pattern-dismiss]")?.dispatchEvent(new Event("pointerdown"));
    await nextTick();
    expect(trigger.attributes("aria-expanded")).toBe("false");

    wrapper.unmount();
  });
});

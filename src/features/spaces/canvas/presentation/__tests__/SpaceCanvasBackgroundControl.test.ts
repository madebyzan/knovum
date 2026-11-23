import { mount } from "@vue/test-utils";
import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { nextTick } from "vue";

import { i18n } from "../../../../../foundation/i18n/instance";
import { usePreferencesStore } from "../../../../../foundation/preferences/preferences-store";
import { useSpaceStore } from "../../../session/space-store";
import SpaceCanvasBackgroundControl from "../SpaceCanvasBackgroundControl.vue";

vi.mock("../../../enrichment/use-enrichment-job-observation", () => ({
  useEnrichmentJobObservation: () => {},
}));

describe("SpaceCanvasBackgroundControl", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
  });

  it("updates current Space pattern override, resets to global default, and updates global pattern", async () => {
    const store = useSpaceStore();
    const preferences = usePreferencesStore();
    const globalPatternSpy = vi.spyOn(preferences, "setCanvasBackgroundPattern").mockResolvedValue();

    const wrapper = mount(SpaceCanvasBackgroundControl, {
      attachTo: document.body,
      global: { plugins: [i18n] },
    });

    await wrapper.get("button").trigger("click");
    await nextTick();

    const spaceRadios = Array.from(
      document.body.querySelectorAll<HTMLButtonElement>("[data-pattern-scope='space'] [role='radio']"),
    );
    const globalRadios = Array.from(
      document.body.querySelectorAll<HTMLButtonElement>("[data-pattern-scope='global'] [role='radio']"),
    );
    const spaceGridButton = spaceRadios.find((button) => button.textContent?.includes("Grid"));
    const globalNoneButton = globalRadios.find((button) => button.textContent?.includes("None"));
    expect(spaceGridButton).toBeDefined();
    expect(globalNoneButton).toBeDefined();

    spaceGridButton!.click();
    await nextTick();
    expect(store.backgroundPattern).toBe("grid");

    const followGlobalButton = document.body.querySelector<HTMLButtonElement>(
      "[data-action='follow-global-pattern']",
    );
    expect(followGlobalButton).not.toBeNull();
    followGlobalButton!.click();
    await nextTick();
    expect(store.backgroundPattern).toBeUndefined();

    globalNoneButton!.click();
    await nextTick();
    expect(globalPatternSpy).toHaveBeenCalledWith("none");

    wrapper.unmount();
  });
});

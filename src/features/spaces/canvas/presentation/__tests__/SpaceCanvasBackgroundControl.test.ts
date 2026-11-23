import { mount } from "@vue/test-utils";
import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { nextTick } from "vue";

import { i18n } from "../../../../../foundation/i18n/instance";
import { usePreferencesStore } from "../../../../../foundation/preferences/preferences-store";
import SpaceCanvasBackgroundControl from "../SpaceCanvasBackgroundControl.vue";

describe("SpaceCanvasBackgroundControl", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
  });

  it("updates canvas background pattern and curated backdrop through the preferences store", async () => {
    const preferences = usePreferencesStore();
    const patternSpy = vi.spyOn(preferences, "setCanvasBackgroundPattern").mockResolvedValue();
    const backdropSpy = vi.spyOn(preferences, "setCanvasBackdrop").mockResolvedValue();

    const wrapper = mount(SpaceCanvasBackgroundControl, {
      attachTo: document.body,
      global: { plugins: [i18n] },
    });

    await wrapper.get("button").trigger("click");
    await nextTick();

    const radios = Array.from(document.body.querySelectorAll<HTMLButtonElement>("[role='radio']"));
    const noneButton = radios.find((button) => button.textContent?.includes("None"));
    const summerGlowButton = radios.find((button) => button.textContent?.includes("Summer Glow"));
    expect(noneButton).toBeDefined();
    expect(summerGlowButton).toBeDefined();

    noneButton!.click();
    summerGlowButton!.click();
    await nextTick();

    expect(patternSpy).toHaveBeenCalledWith("none");
    expect(backdropSpy).toHaveBeenCalledWith("summer-glow");
    wrapper.unmount();
  });
});

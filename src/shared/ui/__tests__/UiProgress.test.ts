import { mount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";

import UiProgress from "../UiProgress.vue";

describe("UiProgress", () => {
  it("uses one clamped value for visual and accessible progress", () => {
    const wrapper = mount(UiProgress, { props: { label: "Upload", value: 140 } });
    const progress = wrapper.get("[role='progressbar']");

    expect(progress.attributes("aria-valuenow")).toBe("100");
    expect(progress.get("div").attributes("style")).toContain("width: 100%");
  });

  it("clamps negative progress to zero", () => {
    const wrapper = mount(UiProgress, { props: { label: "Upload", value: -5 } });
    expect(wrapper.get("[role='progressbar']").attributes("aria-valuenow")).toBe("0");
  });

  it("reports a custom max in caller units and fills proportionally", () => {
    const wrapper = mount(UiProgress, { props: { label: "Upload", value: 2, max: 8 } });
    const progress = wrapper.get("[role='progressbar']");

    expect(progress.attributes("aria-valuenow")).toBe("2");
    expect(progress.attributes("aria-valuemax")).toBe("8");
    expect(progress.get("div").attributes("style")).toContain("width: 25%");
  });
});

import { mount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";

import UiPanelToggleIcon from "../UiPanelToggleIcon.vue";

function mountIcon(side: "left" | "right", open: boolean) {
  return mount(UiPanelToggleIcon, { props: { side, open } });
}

describe("panel toggle icon", () => {
  it("draws one geometry for both states and only changes the pane's fill state", async () => {
    const wrapper = mountIcon("left", false);
    const markup = () => wrapper.findAll("path, rect").map((element) => element.attributes("d") ?? "frame");
    const closed = markup();
    expect(wrapper.attributes("data-open")).toBeUndefined();

    await wrapper.setProps({ open: true });

    // No glyph swap: the same elements stay mounted, so nothing cross-fades.
    expect(markup()).toEqual(closed);
    expect(wrapper.attributes("data-open")).toBe("true");
    wrapper.unmount();
  });

  it("mirrors the pane and its divider to the side the panel lives on", () => {
    const left = mountIcon("left", true);
    const right = mountIcon("right", true);

    expect(left.get(".ui-panel-toggle-icon-pane").attributes("d")).toMatch(/^M9 3H5/);
    expect(right.get(".ui-panel-toggle-icon-pane").attributes("d")).toMatch(/^M15 3h4/);
    expect(left.findAll("path").at(-1)!.attributes("d")).toBe("M9 3v18");
    expect(right.findAll("path").at(-1)!.attributes("d")).toBe("M15 3v18");
    expect(right.attributes("data-side")).toBe("right");
    left.unmount();
    right.unmount();
  });

  it("stays decorative, leaving the label and expanded state to the button", () => {
    const wrapper = mountIcon("right", false);

    expect(wrapper.attributes("aria-hidden")).toBe("true");
    wrapper.unmount();
  });
});

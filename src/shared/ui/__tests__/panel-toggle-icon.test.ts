import { mount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";

import UiPanelToggleIcon from "../UiPanelToggleIcon.vue";

function mountIcon(side: "left" | "right", open: boolean) {
  return mount(UiPanelToggleIcon, { props: { side, open } });
}

describe("panel toggle icon", () => {
  it("keeps one set of elements across states, so opening changes the fill and nothing else", async () => {
    const wrapper = mountIcon("left", false);
    const shapes = () => wrapper.findAll("rect").map((rect) => rect.element);
    const closed = shapes();
    expect(wrapper.attributes("data-open")).toBeUndefined();

    await wrapper.setProps({ open: true });
    expect(wrapper.attributes("data-open")).toBe("true");
    expect(shapes()).toEqual(closed);
    wrapper.unmount();
  });

  it("insets the pane from the frame on every side, with a fill covering exactly its outline", () => {
    const wrapper = mountIcon("left", true);
    const [frame, outline, fill] = wrapper.findAll("rect").map((rect) => {
      const number = (name: string) => Number(rect.attributes(name));
      const half = Number(rect.attributes("stroke-width") ?? wrapper.attributes("stroke-width")) / 2;
      const stroked = rect.attributes("stroke") !== "none";
      const grow = stroked ? half : 0;
      return {
        left: number("x") - grow,
        top: number("y") - grow,
        right: number("x") + number("width") + grow,
        bottom: number("y") + number("height") + grow,
        innerLeft: number("x") + grow,
        innerTop: number("y") + grow,
        innerBottom: number("y") + number("height") - grow,
      };
    });
    const inset = [
      fill!.left - frame!.innerLeft,
      fill!.top - frame!.innerTop,
      frame!.innerBottom - fill!.bottom,
    ];

    expect(new Set(inset).size).toBe(1);
    expect(inset[0]).toBeGreaterThan(1);
    expect({ ...fill, innerLeft: 0, innerTop: 0, innerBottom: 0 }).toEqual({
      ...outline,
      innerLeft: 0,
      innerTop: 0,
      innerBottom: 0,
    });
    wrapper.unmount();
  });

  it("mirrors the pane to the window edge its sidebar belongs to", () => {
    const left = mountIcon("left", true);
    const right = mountIcon("right", true);

    expect(left.get("g").attributes("transform")).toBeUndefined();
    expect(right.get("g").attributes("transform")).toBe("matrix(-1 0 0 1 24 0)");
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

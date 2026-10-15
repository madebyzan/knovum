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

  it("draws the pane solid in both states, inset evenly from the frame", async () => {
    const wrapper = mountIcon("left", false);
    const [frame, pane] = wrapper.findAll("rect");
    const number = (rect: typeof frame, name: string) => Number(rect!.attributes(name));
    const frameInner = number(frame, "stroke-width") || Number(wrapper.attributes("stroke-width"));
    const inner = {
      left: number(frame, "x") + frameInner / 2,
      top: number(frame, "y") + frameInner / 2,
      bottom: number(frame, "y") + number(frame, "height") - frameInner / 2,
    };
    const inset = [
      number(pane, "x") - inner.left,
      number(pane, "y") - inner.top,
      inner.bottom - number(pane, "y") - number(pane, "height"),
    ];

    expect(wrapper.findAll("rect")).toHaveLength(2);
    expect(new Set(inset).size).toBe(1);
    expect(inset[0]).toBeGreaterThan(1);
    // The width is the state, set by the stylesheet so it can animate; an attribute would pin it.
    const expectSolidPane = () => {
      expect(pane!.attributes("fill")).toBe("currentColor");
      expect(pane!.attributes("stroke")).toBe("none");
      expect(pane!.attributes("width")).toBeUndefined();
    };
    expectSolidPane();
    await wrapper.setProps({ open: true });
    expectSolidPane();
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

import { mount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";

import UiMarkdown from "../UiMarkdown.vue";

describe("UiMarkdown citations", () => {
  it("renders only issued markers, and only openable ones as buttons", () => {
    const wrapper = mount(UiMarkdown, {
      props: {
        content: "Kept [C1] and stale [C2]; unknown [C9], code `[C1]`.",
        citations: new Map([
          [1, true],
          [2, false],
        ]),
      },
    });
    const button = wrapper.get("button.ui-citation");
    expect(button.text()).toBe("1");
    expect(wrapper.get("span.ui-citation").text()).toBe("2");
    expect(wrapper.text()).toContain("[C9]");
    expect(wrapper.get("code").text()).toBe("[C1]");
  });

  it("emits the cited index when a citation is clicked", async () => {
    const wrapper = mount(UiMarkdown, {
      props: { content: "Claim [C3].", citations: new Map([[3, true]]) },
    });
    await wrapper.get("button.ui-citation").trigger("click");
    expect(wrapper.emitted("cite")).toEqual([[3]]);
  });

  it("leaves markers as text without citations", () => {
    const wrapper = mount(UiMarkdown, { props: { content: "Claim [C3]." } });
    expect(wrapper.find(".ui-citation").exists()).toBe(false);
    expect(wrapper.text()).toBe("Claim [C3].");
  });
});

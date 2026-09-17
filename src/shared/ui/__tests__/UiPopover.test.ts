import { mount } from "@vue/test-utils";
import { afterEach, describe, expect, it } from "vitest";

import UiPopover from "../UiPopover.vue";

describe("UiPopover", () => {
  let wrapper: ReturnType<typeof mount> | undefined;

  afterEach(() => {
    wrapper?.unmount();
    document.body.innerHTML = "";
  });

  it("renders trigger slot element", () => {
    wrapper = mount(UiPopover, {
      attachTo: document.body,
      slots: {
        trigger: "<button>Open Popover</button>",
        default: "Popover Content",
      },
    });
    expect(wrapper.get("button").text()).toBe("Open Popover");
  });
});

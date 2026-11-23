import { mount } from "@vue/test-utils";
import { afterEach, describe, expect, it, vi } from "vitest";
import { nextTick } from "vue";

import UiIconButton from "../UiIconButton.vue";
import UiPopover from "../UiPopover.vue";
import UiTooltip from "../UiTooltip.vue";

describe("UiTooltip", () => {
  afterEach(() => {
    vi.useRealTimers();
    document.body.innerHTML = "";
  });

  it("keeps the trigger element when the tooltip is disabled at runtime", async () => {
    const wrapper = mount(UiTooltip, {
      attachTo: document.body,
      props: { text: "Background", disabled: false },
      slots: { default: '<button type="button">Background</button>' },
    });

    const trigger = wrapper.get("button").element;
    await wrapper.setProps({ disabled: true });
    await nextTick();

    // An anchor-based popover or tooltip measures this element, so replacing it on a state
    // change leaves the popper pointing at a detached node.
    expect(wrapper.get("button").element).toBe(trigger);
    expect(trigger.isConnected).toBe(true);

    wrapper.unmount();
  });

  it("hides an open tooltip when it becomes disabled", async () => {
    vi.useFakeTimers();
    const wrapper = mount(UiTooltip, {
      attachTo: document.body,
      props: { text: "Background", disabled: false },
      slots: { default: '<button type="button">Background</button>' },
    });

    await wrapper.get("button").trigger("pointermove");
    vi.advanceTimersByTime(400);
    await nextTick();
    expect(wrapper.get("button").attributes("data-state")).not.toBe("closed");

    await wrapper.setProps({ disabled: true });
    await nextTick();
    expect(wrapper.get("button").attributes("data-state")).toBe("closed");

    wrapper.unmount();
  });

  it("keeps an outer popover in charge of the shared trigger element", async () => {
    const wrapper = mount(
      {
        components: { UiIconButton, UiPopover },
        template: `
          <UiPopover v-model:open="open">
            <template #trigger>
              <UiIconButton label="Background" :tooltip="false" />
            </template>
            <p>panel</p>
          </UiPopover>`,
        data: () => ({ open: false }),
      },
      { attachTo: document.body },
    );

    await wrapper.get("button").trigger("click");
    await nextTick();

    // Both primitives mark the element they wrap with data-state, and the app styles popper
    // triggers with data-[state=open]; a disabled tooltip must not take that over.
    const trigger = wrapper.get("button");
    expect(trigger.attributes("data-state")).toBe("open");
    expect(trigger.attributes("aria-expanded")).toBe("true");

    wrapper.unmount();
  });
});

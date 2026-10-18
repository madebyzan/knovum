import { mount } from "@vue/test-utils";
import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { i18n } from "../../../../../foundation/i18n/instance";
import { workspaceRightSidebarKey } from "../../../../../shared/contracts/workspace-right-sidebar";
import { openStoreSpaceTab } from "../../../__tests__/support/session-context";
import { useSpaceStore } from "../../../session/space-store";
import SpaceRightSidebarViewPicker from "../SpaceRightSidebarViewPicker.vue";

describe("SpaceRightSidebarViewPicker", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
  });

  it("renders segmented tabs for Ask and Inspect and switches views on click", async () => {
    const store = useSpaceStore();
    openStoreSpaceTab(store, "inbox");
    const rightSidebar = { open: vi.fn<() => void>(), close: vi.fn<() => void>() };
    const wrapper = mount(SpaceRightSidebarViewPicker, {
      global: {
        plugins: [i18n],
        provide: {
          [workspaceRightSidebarKey as symbol]: rightSidebar,
        },
      },
    });

    const buttons = wrapper.findAll('button[role="tab"]');
    expect(buttons).toHaveLength(2);

    expect(buttons[0]?.attributes("aria-selected")).toBe("true");
    expect(buttons[1]?.attributes("aria-selected")).toBe("false");

    await buttons[1]?.trigger("click");
    expect(store.rightSidebarView).toBe("inspect");
    expect(rightSidebar.open).toHaveBeenCalled();

    await buttons[0]?.trigger("click");
    expect(store.rightSidebarView).toBe("ask");
  });
});

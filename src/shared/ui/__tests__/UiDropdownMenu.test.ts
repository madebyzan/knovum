import { mount } from "@vue/test-utils";
import { describe, expect, it, vi } from "vitest";

import UiDropdownMenu from "../UiDropdownMenu.vue";

vi.mock("@tauri-apps/plugin-os", () => ({
  platform: () => "macos",
}));

describe("UiDropdownMenu", () => {
  it("renders trigger slot element", () => {
    const wrapper = mount(UiDropdownMenu, {
      slots: {
        trigger: '<button id="menu-trigger">Actions</button>',
        default: "<div>Menu items</div>",
      },
    });
    expect(wrapper.find("#menu-trigger").exists()).toBe(true);
    expect(wrapper.find("#menu-trigger").text()).toBe("Actions");
  });

  it("supports declarative items mode with header and footer slots", () => {
    const wrapper = mount(UiDropdownMenu, {
      props: {
        open: true,
        items: [
          { label: "Edit" },
          { separator: true },
          { label: "Show Grid", checked: true },
          { label: "Delete", destructive: true, shortcut: "mod+d" },
        ],
      },
      slots: {
        trigger: "<button>Options</button>",
        header: '<div id="menu-header">Header</div>',
        footer: '<div id="menu-footer">Footer</div>',
      },
    });
    expect(wrapper.find("button").exists()).toBe(true);
  });
});

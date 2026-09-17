import { mount } from "@vue/test-utils";
import { describe, expect, it, vi } from "vitest";

import UiKbd from "../UiKbd.vue";

vi.mock("@tauri-apps/plugin-os", () => ({
  platform: () => "macos",
}));

describe("UiKbd", () => {
  it("renders subtle text shortcut by default", () => {
    const wrapper = mount(UiKbd, {
      props: {
        shortcut: "$mod+k",
      },
    });
    expect(wrapper.text()).toBe("⌘K");
    expect(wrapper.find("kbd").exists()).toBe(true);
  });

  it("renders badge variant with separate kbd elements", () => {
    const wrapper = mount(UiKbd, {
      props: {
        shortcut: "$mod+shift+p",
        variant: "badge",
      },
    });
    const kbds = wrapper.findAll("kbd");
    expect(kbds).toHaveLength(3);
    expect(kbds[0]?.text()).toBe("⌘");
    expect(kbds[1]?.text()).toBe("⇧");
    expect(kbds[2]?.text()).toBe("P");
  });

  it("renders default slot when no shortcut is passed", () => {
    const wrapper = mount(UiKbd, {
      slots: {
        default: "Esc",
      },
    });
    expect(wrapper.text()).toBe("Esc");
  });
});

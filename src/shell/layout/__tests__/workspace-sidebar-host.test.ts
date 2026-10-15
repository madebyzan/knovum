import { mount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";
import { defineComponent, h, markRaw } from "vue";

import { i18n } from "../../../foundation/i18n/instance";
import type { WorkspaceDefinition } from "../../../shared/contracts/workspace-definition";
import WorkspaceLayout from "../WorkspaceLayout.vue";

const workspace = {
  id: "spaces",
  view: markRaw(defineComponent({ name: "WorkspaceView", setup: () => () => h("div", "work area") })),
  sidebar: markRaw(defineComponent({ name: "WorkspaceSidebarView", setup: () => () => h("div", "sidebar") })),
  rightSidebarLayout: { minSize: 240, maxSize: 480 },
} as unknown as WorkspaceDefinition;

function mountLayout(sidebarPinned: boolean) {
  return mount(WorkspaceLayout, {
    props: {
      workspace,
      sidebarPinned,
      sidebarWidth: 280,
      sidebarPreview: false,
      rightSidebarOpen: false,
      rightSidebarContentMounted: false,
      rightSidebarWidth: 320,
    },
    global: { plugins: [i18n] },
  });
}

describe("workspace sidebar host", () => {
  it("keeps one host across pinning so the subtree is never moved", async () => {
    const wrapper = mountLayout(false);
    const surface = wrapper.get(".workspace-sidebar-surface");
    const sidebar = wrapper.getComponent({ name: "WorkspaceSidebar" });
    expect(surface.element.contains(sidebar.element)).toBe(true);

    await wrapper.setProps({ sidebarPinned: true });

    // Same element, not a re-created one: a moved subtree would reset scroll and re-measure every row.
    expect(wrapper.get(".workspace-sidebar-surface").element).toBe(surface.element);
    expect(wrapper.getComponent({ name: "WorkspaceSidebar" }).element).toBe(sidebar.element);
    expect(surface.element.contains(sidebar.element)).toBe(true);
    wrapper.unmount();
  });

  it("precedes the splitter so focus order starts at the sidebar", () => {
    const wrapper = mountLayout(true);
    const children = [...wrapper.element.children];
    const surfaceIndex = children.findIndex((child) => child.classList.contains("workspace-sidebar-surface"));
    const splitterIndex = children.findIndex((child) => child.classList.contains("workspace-splitter-group"));

    expect(surfaceIndex).toBeGreaterThanOrEqual(0);
    expect(surfaceIndex).toBeLessThan(splitterIndex);
    wrapper.unmount();
  });

  it("carries the floating role only while previewing without a pin", async () => {
    const wrapper = mountLayout(false);
    const surface = wrapper.get(".workspace-sidebar-surface");
    expect(surface.classes()).not.toContain("is-visible");

    await wrapper.setProps({ sidebarPreview: true });
    expect(surface.classes()).toContain("is-visible");
    expect(surface.classes()).toContain("is-preview");

    await wrapper.setProps({ sidebarPinned: true });
    expect(surface.classes()).toContain("is-visible");
    expect(surface.classes()).not.toContain("is-preview");
    wrapper.unmount();
  });
});

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

function mountLayout(props: {
  sidebarPinned: boolean;
  sidebarPreview?: boolean;
  rightSidebarOpen?: boolean;
}) {
  return mount(WorkspaceLayout, {
    props: {
      workspace,
      sidebarPinned: props.sidebarPinned,
      sidebarWidth: 280,
      sidebarPreview: props.sidebarPreview ?? false,
      rightSidebarOpen: props.rightSidebarOpen ?? false,
      rightSidebarContentMounted: props.rightSidebarOpen ?? false,
      rightSidebarWidth: 320,
    },
    global: { plugins: [i18n] },
  });
}

describe("workspace layout", () => {
  it("keeps one sidebar host across pinning so the subtree is never moved", async () => {
    const wrapper = mountLayout({ sidebarPinned: false });
    const surface = wrapper.get(".workspace-sidebar-surface");
    const sidebar = wrapper.getComponent({ name: "WorkspaceSidebar" });
    expect(surface.element.contains(sidebar.element)).toBe(true);

    await wrapper.setProps({ sidebarPinned: true });

    // Same element, not a re-created one: a moved subtree would reset scroll and re-measure every row.
    expect(wrapper.get(".workspace-sidebar-surface").element).toBe(surface.element);
    expect(wrapper.getComponent({ name: "WorkspaceSidebar" }).element).toBe(sidebar.element);
    wrapper.unmount();
  });

  it("is the sized region when pinned and leaves the flow when previewing", async () => {
    const wrapper = mountLayout({ sidebarPinned: true });
    const surface = wrapper.get(".workspace-sidebar-surface");
    expect(surface.classes()).toContain("is-pinned");
    expect(surface.classes()).toContain("is-visible");

    await wrapper.setProps({ sidebarPinned: false, sidebarPreview: true });
    expect(surface.classes()).not.toContain("is-pinned");
    expect(surface.classes()).toContain("is-visible");

    await wrapper.setProps({ sidebarPreview: false });
    expect(surface.classes()).not.toContain("is-visible");
    wrapper.unmount();
  });

  it("carries the sidebar width on the host that is sized by it", () => {
    const wrapper = mountLayout({ sidebarPinned: true });

    expect(wrapper.get(".workspace-sidebar-surface").attributes("style")).toContain(
      "--workspace-sidebar-width: 280px",
    );
    wrapper.unmount();
  });

  it("offers a sash only for a sidebar that owns layout width", async () => {
    const wrapper = mountLayout({ sidebarPinned: false });
    expect(wrapper.findAllComponents({ name: "WorkspaceSidebarSash" })).toHaveLength(0);

    await wrapper.setProps({ sidebarPinned: true });
    const sashes = wrapper.findAllComponents({ name: "WorkspaceSidebarSash" });
    expect(sashes).toHaveLength(1);
    expect(sashes[0]!.props("controls")).toBe("spaces-sidebar");
    expect(sashes[0]!.props("edge")).toBe("leading");

    await wrapper.setProps({ rightSidebarOpen: true, rightSidebarContentMounted: true });
    expect(wrapper.findAllComponents({ name: "WorkspaceSidebarSash" })).toHaveLength(2);
    wrapper.unmount();
  });

  it("reports a sash width through the region it resizes", async () => {
    const wrapper = mountLayout({ sidebarPinned: true });
    const sash = wrapper.getComponent({ name: "WorkspaceSidebarSash" });

    sash.vm.$emit("resize", 240);
    sash.vm.$emit("commit");
    await wrapper.vm.$nextTick();

    expect(wrapper.emitted("set-sidebar-width")).toEqual([[240]]);
    expect(wrapper.emitted("commit-layout")).toEqual([[]]);
    wrapper.unmount();
  });

  it("keeps a hidden region out of the accessibility tree", async () => {
    const wrapper = mountLayout({ sidebarPinned: false });
    const pane = wrapper.get("#spaces-right-sidebar");
    expect(pane.attributes("aria-hidden")).toBe("true");
    expect(pane.classes()).not.toContain("is-visible");

    await wrapper.setProps({ rightSidebarOpen: true, rightSidebarContentMounted: true });
    expect(pane.attributes("aria-hidden")).toBe("false");
    expect(pane.classes()).toContain("is-visible");
    wrapper.unmount();
  });
});

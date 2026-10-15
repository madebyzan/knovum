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

/** The panel clock at rest: every edge is where its state puts it. */
function restingEdges(pinned: boolean, visible: boolean, rightOpen: boolean) {
  return { leading: pinned ? 280 : 0, sidebar: visible ? 280 : 0, trailing: rightOpen ? 320 : 0 };
}

function mountLayout(props: {
  sidebarPinned: boolean;
  sidebarPreview?: boolean;
  rightSidebarOpen?: boolean;
}) {
  const preview = props.sidebarPreview ?? false;
  return mount(WorkspaceLayout, {
    props: {
      workspace,
      sidebarPinned: props.sidebarPinned,
      sidebarWidth: 280,
      sidebarPreview: preview,
      rightSidebarOpen: props.rightSidebarOpen ?? false,
      rightSidebarContentMounted: props.rightSidebarOpen ?? false,
      rightSidebarWidth: 320,
      edges: restingEdges(
        props.sidebarPinned,
        props.sidebarPinned || preview,
        props.rightSidebarOpen ?? false,
      ),
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

  it("reserves width when pinned and floats over the work area when previewing", async () => {
    const wrapper = mountLayout({ sidebarPinned: true });
    const slot = () => wrapper.get(".workspace-sidebar-slot").attributes("style");
    const surface = wrapper.get(".workspace-sidebar-surface");
    const offset = () => (surface.element as HTMLElement).style.transform;
    expect(slot()).toBe("--workspace-leading-edge: 280px;");
    expect(offset()).toBe("translate3d(0px, 0, 0)");
    expect(surface.classes()).not.toContain("is-floating");
    expect(surface.attributes("aria-hidden")).toBe("false");

    await wrapper.setProps({
      sidebarPinned: false,
      sidebarPreview: true,
      edges: restingEdges(false, true, false),
    });
    expect(slot()).toBe("--workspace-leading-edge: 0px;");
    expect(offset()).toBe("translate3d(0px, 0, 0)");
    expect(surface.classes()).toContain("is-floating");
    expect(surface.attributes("aria-hidden")).toBe("false");

    await wrapper.setProps({ sidebarPreview: false, edges: restingEdges(false, false, false) });
    expect(offset()).toBe("translate3d(-280px, 0, 0)");
    expect(surface.classes()).not.toContain("is-floating");
    expect(surface.attributes("aria-hidden")).toBe("true");
    wrapper.unmount();
  });

  it("places the slot, the sidebar and the trailing pane from one frame of the panel clock", async () => {
    const wrapper = mountLayout({ sidebarPinned: true, rightSidebarOpen: true });
    await wrapper.setProps({ edges: { leading: 131, sidebar: 131, trailing: 47 } });

    expect(wrapper.get(".workspace-sidebar-slot").attributes("style")).toBe(
      "--workspace-leading-edge: 131px;",
    );
    expect((wrapper.get("#spaces-sidebar").element as HTMLElement).style.transform).toBe(
      "translate3d(-149px, 0, 0)",
    );
    expect((wrapper.get("#spaces-right-sidebar").element as HTMLElement).style.width).toBe("47px");
    wrapper.unmount();
  });

  it("publishes both sidebar widths to the regions sized by them", () => {
    const wrapper = mountLayout({ sidebarPinned: true });
    const style = wrapper.attributes("style");

    expect(style).toContain("--workspace-sidebar-width: 280px");
    expect(style).toContain("--workspace-right-sidebar-width: 320px");
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
    expect(pane.attributes("inert")).toBeDefined();

    await wrapper.setProps({ rightSidebarOpen: true, rightSidebarContentMounted: true });
    expect(pane.attributes("aria-hidden")).toBe("false");
    expect(pane.attributes("inert")).toBeUndefined();
    wrapper.unmount();
  });
});

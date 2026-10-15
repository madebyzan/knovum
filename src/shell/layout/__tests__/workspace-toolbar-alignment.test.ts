import { mount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";
import { computed, defineComponent, h, markRaw } from "vue";

import { i18n } from "../../../foundation/i18n/instance";
import type { WorkspaceDefinition } from "../../../shared/contracts/workspace-definition";
import WorkspaceToolbar from "../WorkspaceToolbar.vue";

const empty = markRaw(defineComponent({ setup: () => () => h("div") }));
const workspace = {
  id: "spaces",
  toolbar: markRaw(defineComponent({ name: "TabStrip", setup: () => () => h("div", "tabs") })),
  toolbarLeading: empty,
  rightSidebarAvailable: computed(() => true),
} as unknown as WorkspaceDefinition;

function mountToolbar(sidebarPinned: boolean, trailing = 0) {
  return mount(WorkspaceToolbar, {
    props: {
      workspace,
      sidebarPinned,
      rightSidebarOpen: trailing > 0,
      edges: { leading: sidebarPinned ? 240 : 0, sidebar: sidebarPinned ? 240 : 0, trailing },
    },
    global: {
      plugins: [i18n],
      stubs: { TaskCenter: true, WorkspaceNavigationControls: true, UiIconButton: true },
    },
  });
}

function columns(wrapper: ReturnType<typeof mountToolbar>) {
  return (wrapper.get(".workspace-toolbar-content").element as HTMLElement).style.gridTemplateColumns;
}

describe("workspace toolbar alignment", () => {
  it("ends the leading column where a pinned sidebar ends, so the first tab meets the canvas edge", () => {
    const wrapper = mountToolbar(true);

    expect(columns(wrapper)).toBe(
      "minmax(auto, max(var(--knovum-workspace-gutter), 240px)) minmax(0, 1fr) 96px",
    );
    wrapper.unmount();
  });

  it("follows the work area gutter without a sidebar, never narrower than the navigation cluster", () => {
    const wrapper = mountToolbar(false);

    expect(columns(wrapper)).toBe(
      "minmax(auto, max(var(--knovum-workspace-gutter), 0px)) minmax(0, 1fr) 96px",
    );
    expect(wrapper.get(".workspace-toolbar-content > div").classes()).toContain("min-w-max");
    wrapper.unmount();
  });

  it("adds no leading inset of its own in front of the tabs", () => {
    const wrapper = mountToolbar(true);
    const tabColumn = wrapper.getComponent({ name: "TabStrip" }).element.parentElement!;

    expect([...tabColumn.classList].some((name) => /^(?:pl|px)-/.test(name))).toBe(false);
    wrapper.unmount();
  });

  it("takes every column from the panel clock's frame, with no transition of its own", async () => {
    const wrapper = mountToolbar(true);
    await wrapper.setProps({ edges: { leading: 117, sidebar: 117, trailing: 0 } });

    expect(columns(wrapper)).toBe(
      "minmax(auto, max(var(--knovum-workspace-gutter), 117px)) minmax(0, 1fr) 96px",
    );
    wrapper.unmount();
  });

  it("reserves the trailing column the right pane occupies", () => {
    const wrapper = mountToolbar(true, 320);

    expect(columns(wrapper)).toBe(
      "minmax(auto, max(var(--knovum-workspace-gutter), 240px)) minmax(0, 1fr) 320px",
    );
    wrapper.unmount();
  });
});

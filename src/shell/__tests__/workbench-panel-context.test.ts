import { mount } from "@vue/test-utils";
import { expect, it, onTestFinished, vi } from "vitest";
import { computed, defineComponent, h, nextTick, shallowRef } from "vue";

import { i18n } from "../../foundation/i18n/instance";
import type { WorkspaceDefinition } from "../../shared/contracts/workspace-definition";
import WorkspaceToolbar from "../components/WorkspaceToolbar.vue";
import { useShellController } from "../use-shell-controller";

vi.mock("@tauri-apps/api/event", () => ({ listen: async () => () => {} }));
vi.mock("@tauri-apps/plugin-os", () => ({ platform: () => "macos" }));
vi.mock("@tauri-apps/plugin-log", () => ({ error: async () => {} }));
vi.mock("../../foundation/preferences/preferences-store", () => ({
  usePreferencesStore: () => ({
    workbench: { sidebarVisible: true, sidebarWidth: 260, rightSidebarVisible: true, rightSidebarWidth: 380 },
    setWorkbenchLayout: async () => {},
  }),
}));

it("hides unavailable panel controls while retaining the user's requested layout", async () => {
  const available = shallowRef(true);
  const Empty = defineComponent(() => () => null);
  const workspace: WorkspaceDefinition = {
    view: Empty,
    sidebar: Empty,
    rightSidebar: Empty,
    toolbar: Empty,
    toolbarLeading: Empty,
    rightSidebarAvailable: computed(() => available.value),
    rightSidebarLayout: { defaultSize: 380, minSize: 300, maxSize: 480 },
    openDefaultRightSidebar: () => {},
    useCommands: () => computed(() => []),
  };
  let controller!: ReturnType<typeof useShellController>;
  const wrapper = mount(
    {
      setup() {
        controller = useShellController(workspace, shallowRef(true), () => {});
        return () =>
          h(WorkspaceToolbar, {
            workspace,
            sidebarPinned: true,
            sidebarWidth: 260,
            rightSidebarOpen: controller.rightSidebarOpen.value,
            rightSidebarLayoutWidth: controller.rightSidebarOpen.value ? 380 : 0,
          });
      },
    },
    {
      global: {
        plugins: [i18n],
        stubs: { TaskCenter: true, WorkspaceNavigationControls: true, UiIconButton: true },
      },
    },
  );
  onTestFinished(() => wrapper.unmount());
  expect(controller.rightSidebarOpen.value).toBe(true);
  expect(wrapper.find("#workspace-right-sidebar-toggle").exists()).toBe(true);
  available.value = false;
  await nextTick();
  expect(controller.rightSidebarOpen.value).toBe(false);
  expect(wrapper.find("#workspace-right-sidebar-toggle").exists()).toBe(false);
  available.value = true;
  await nextTick();
  expect(controller.rightSidebarOpen.value).toBe(true);
  expect(wrapper.find("#workspace-right-sidebar-toggle").exists()).toBe(true);
});

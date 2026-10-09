import { flushPromises, mount } from "@vue/test-utils";
import { createPinia } from "pinia";
import { expect, it, vi } from "vitest";
import { defineComponent, h, reactive, shallowRef } from "vue";

import { useContextDock } from "../../features/spaces/canvas/use-context-dock";
import { useSpacesWorkspace } from "../../features/spaces/use-spaces-workspace";
import { i18n } from "../../foundation/i18n/i18n";
import type { WorkspaceDefinition } from "../../shared/contracts/workspace-definition";
import { useShellController } from "../../shell/use-shell-controller";

vi.mock("@tauri-apps/api/event", () => ({ listen: async () => () => {} }));
vi.mock("@tauri-apps/plugin-os", () => ({ platform: () => "macos" }));
vi.mock("@tauri-apps/plugin-log", () => ({ error: async () => {} }));
vi.mock("../../features/spaces/session/space-store", () => ({ useSpaceStore: () => store }));
vi.mock("../../foundation/preferences/preferences-store", () => ({
  usePreferencesStore: () => ({
    workbench: {
      sidebarVisible: true,
      sidebarWidth: 260,
      rightSidebarVisible: false,
      rightSidebarWidth: 380,
    },
    setWorkbenchLayout: async () => {},
  }),
}));

const store = reactive({
  workspaceMode: "library",
  activeSpaceId: undefined,
  activeSpace: undefined,
  openSpaceIds: [],
  closedSpaceIds: [],
  nodes: [],
  scenes: [],
  workflowMode: undefined,
  focusSpaceLibrarySearch: vi.fn<() => void>(),
  setRightSidebarView: vi.fn<(view: string) => void>(),
});

it("composes feature commands after shell operations exist, with context available to descendants", async () => {
  const warnings = vi.fn<(message: string) => void>();
  let workspace!: WorkspaceDefinition;
  const Dock = defineComponent({
    setup() {
      const dock = useContextDock();
      return () => h("button", { onClick: () => dock.openAsk() }, "Ask");
    },
  });
  const Shell = defineComponent({
    setup() {
      const { commands } = useShellController(workspace, shallowRef(true), () => {});
      return () =>
        h("div", [
          h(Dock),
          h("button", { onClick: () => commands.value.find(({ id }) => id === "spaces.find")?.run() }),
        ]);
    },
  });
  const Root = defineComponent({
    setup() {
      workspace = useSpacesWorkspace();
      return () => h(Shell);
    },
  });
  const wrapper = mount(Root, {
    global: { plugins: [createPinia(), i18n], config: { warnHandler: warnings } },
  });
  try {
    await flushPromises();
    const [ask, find] = wrapper.findAll("button");
    await ask!.trigger("click");
    await find!.trigger("click");
    expect(store.setRightSidebarView).toHaveBeenCalledWith("ask");
    expect(store.focusSpaceLibrarySearch).toHaveBeenCalledOnce();
    expect(warnings.mock.calls.filter(([message]) => message.includes("injection"))).toEqual([]);
  } finally {
    wrapper.unmount();
  }
});

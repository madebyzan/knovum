import { QueryClient, VueQueryPlugin } from "@tanstack/vue-query";
import type { open as openDialog } from "@tauri-apps/plugin-dialog";
import { flushPromises, mount } from "@vue/test-utils";
import { expect, it, vi } from "vitest";

import { i18n } from "../../../foundation/i18n/i18n";
import type { ConnectorDefinition } from "../../../shared/contracts/generated/ConnectorDefinition";
import { settingsLeaveCoordinatorKey } from "../../../shared/contracts/settings-leave";
import PluginConnectorConfiguration from "../connectors/components/PluginConnectorConfiguration.vue";

const mocks = vi.hoisted(() => ({
  invoke: vi.fn<(command: string, payload?: Record<string, unknown>) => Promise<unknown>>(),
  open: vi.fn<typeof openDialog>(),
}));
vi.mock("../../../foundation/ipc/invoke-command", () => ({ invokeCommand: mocks.invoke }));
vi.mock("@tauri-apps/plugin-dialog", () => ({ open: mocks.open }));

it("binds a user-selected installed executable without modifying the publisher declaration", async () => {
  const connector: ConnectorDefinition = {
    id: "service",
    revision: "one",
    name: "Local",
    description: "Reference",
    pluginId: "plugin",
    componentName: "local",
    authorization: { type: "none" },
    endpoint: null,
  };
  mocks.open.mockResolvedValue("/installed/node");
  mocks.invoke.mockImplementation(async (command: string) => {
    if (command === "plugin:extensions|list_plugin_installations")
      return [
        {
          id: "plugin",
          revision: {
            connectors: [
              {
                componentName: "local",
                configuration: { type: "stdio", command: "node", args: ["server.js"] },
              },
            ],
          },
        },
      ];
    if (command === "plugin:extensions|configure_plugin_connector")
      return { value: connector, cleanupErrors: [] };
    throw new Error(`Unexpected command: ${command}`);
  });
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  const wrapper = mount(PluginConnectorConfiguration, {
    props: { connector },
    global: {
      plugins: [i18n, [VueQueryPlugin, { queryClient }]],
      mocks: { $t: (key: string) => key },
      provide: {
        [settingsLeaveCoordinatorKey as symbol]: { register: () => () => {}, requestLeave: async () => {} },
      },
    },
  });
  try {
    await flushPromises();
    expect(wrapper.text()).toContain("extensions.connectors.process.configurationRequired");
    await wrapper.get('[data-testid="bind-plugin-executable"]').trigger("click");
    await flushPromises();
    expect(mocks.open).toHaveBeenCalledWith({ directory: false, multiple: false });
    expect(mocks.invoke).toHaveBeenCalledWith("plugin:extensions|configure_plugin_connector", {
      connectorId: "service",
      revision: "one",
      configuration: { type: "stdio", executable: "/installed/node" },
    });
  } finally {
    wrapper.unmount();
    queryClient.clear();
  }
});

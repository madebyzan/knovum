import { QueryClient, VueQueryPlugin } from "@tanstack/vue-query";
import { flushPromises, mount } from "@vue/test-utils";
import { cloneDeep } from "es-toolkit";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { i18n } from "../../../foundation/i18n/instance";
import type { ConnectorDefinition } from "../../../shared/contracts/generated/ConnectorDefinition";
import type { McpConnection } from "../../../shared/contracts/generated/McpConnection";
import { settingsLeaveCoordinatorKey } from "../../../shared/contracts/settings-leave";
import { resolveConfirmation } from "../../../shared/ui/confirm-action";
import ConnectorServiceSettings from "../connectors/components/ConnectorServiceSettings.vue";
import { connectorFixture, mcpConnectionFixture } from "./support/connector-fixtures";

const mocks = vi.hoisted(() => ({
  invoke: vi.fn<(command: string, input?: Record<string, unknown>) => Promise<unknown>>(),
}));
vi.mock("../../../foundation/ipc/invoke-command", () => ({ invokeCommand: mocks.invoke }));
let services: ConnectorDefinition[];
let connections: McpConnection[];
let mounted: ReturnType<typeof mount> | undefined;
let queryClient: QueryClient;

beforeEach(() => {
  services = [structuredClone(connectorFixture)];
  connections = [mcpConnectionFixture("Work"), mcpConnectionFixture("Personal")];
  queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  mocks.invoke.mockReset();
  mocks.invoke.mockImplementation(async (command: string, input?: Record<string, unknown>) => {
    switch (command) {
      case "plugin:extensions|list_plugin_installations":
        return [];
      case "plugin:extensions|get_agent_extension_availability":
        return {
          plugins: [],
          skills: [],
          connections: [],
          connectors: services,
          diagnostics: [],
          defaultSelection: { pluginIds: [], skillIds: [], connections: [] },
        };
      case "plugin:extensions|list_connector_definitions":
        return structuredClone(services);
      case "plugin:extensions|list_mcp_connections":
        return structuredClone(connections);
      case "plugin:extensions|create_mcp_connection": {
        const created = mcpConnectionFixture("new", input!.name as string);
        connections.push(created);
        return created;
      }
      case "plugin:extensions|delete_mcp_connection": {
        connections = connections.filter((connection) => connection.id !== input!.connectionId);
        return;
      }
      default:
        throw new Error(`Unexpected command: ${command}`);
    }
  });
});
afterEach(() => {
  mounted?.unmount();
  queryClient.clear();
});
async function open() {
  mounted = mount(ConnectorServiceSettings, {
    props: { connectorId: "github" },
    global: {
      plugins: [i18n, [VueQueryPlugin, { queryClient }]],
      provide: {
        [settingsLeaveCoordinatorKey as symbol]: { register: () => () => {}, requestLeave: async () => {} },
      },
      mocks: { $t: (key: string) => key },
    },
  });
  await flushPromises();
  return mounted;
}

describe("service connector management", () => {
  it("creates another connection from one reusable definition without copying endpoints or auth", async () => {
    const wrapper = await open();
    const inspection = wrapper.get('[data-testid="connector-github"]');
    expect(inspection.findAll('[data-testid^="mcp-connection-"]')).toHaveLength(2);
    await inspection.get('[data-testid="add-mcp-connection"]').trigger("click");
    await inspection.get('[data-testid="mcp-connection-name"]').setValue("Research");
    await inspection.get("form").trigger("submit");
    await flushPromises();
    expect(mocks.invoke).toHaveBeenCalledWith("plugin:extensions|create_mcp_connection", {
      connectorId: "github",
      name: "Research",
    });
    expect(services).toHaveLength(1);
    expect(inspection.findAll('[data-testid^="mcp-connection-"]')).toHaveLength(3);
  });

  it("deleting one connection keeps the service and other connections available", async () => {
    const wrapper = await open();
    await wrapper.setProps({ connectorId: "github" });
    await wrapper.get('[data-testid="delete-connection-Work"]').trigger("click");
    expect(mocks.invoke).not.toHaveBeenCalledWith(
      "plugin:extensions|delete_mcp_connection",
      expect.anything(),
    );
    resolveConfirmation(true);
    await flushPromises();
    expect(mocks.invoke).toHaveBeenCalledWith("plugin:extensions|delete_mcp_connection", {
      connectionId: "Work",
      revision: "connection-one",
    });
    expect(wrapper.find('[data-testid="mcp-connection-Work"]').exists()).toBe(false);
    expect(wrapper.find('[data-testid="mcp-connection-Personal"]').exists()).toBe(true);
    expect(wrapper.find('[data-testid="connector-github"]').exists()).toBe(true);
  });

  it("shows committed changes and cleanup failures together after refreshing the service", async () => {
    const original = mocks.invoke.getMockImplementation()!;
    mocks.invoke.mockImplementation(async (command, input) => {
      if (command === "plugin:extensions|update_connector_definition") {
        const definition = {
          ...cloneDeep(input!.definition as ConnectorDefinition),
          revision: "updated",
        };
        services = [definition];
        return { value: definition, cleanupErrors: ["credential-cleanup-failed"] };
      }
      return original(command, input);
    });
    const wrapper = await open();
    await wrapper.setProps({ connectorId: "github" });
    await wrapper.get('[data-testid="edit-connector"]').trigger("click");
    const editor = wrapper.get('[data-testid="connector-editor"]');
    await editor.get('[data-testid="connector-description"]').setValue("Updated purpose");
    await editor.trigger("submit");
    await flushPromises();
    expect(mocks.invoke).toHaveBeenCalledWith("plugin:extensions|update_connector_definition", {
      definition: expect.objectContaining({ description: "Updated purpose" }),
    });
    expect(wrapper.get('[data-testid="connector-github"]').text()).toContain("Updated purpose");
    expect(wrapper.get('[data-testid="extension-cleanup-notice"]').text()).toContain(
      "credential-cleanup-failed",
    );
    expect(wrapper.find('[data-testid="connector-editor"]').exists()).toBe(false);
  });
});

it("configures packaged services without offering individual activation, creation or deletion", async () => {
  services[0]!.pluginId = "owner";
  connections = [mcpConnectionFixture("Packaged")];
  const wrapper = await open();
  expect(wrapper.find('[role="switch"]').exists()).toBe(false);
  expect(wrapper.find('[data-testid="add-mcp-connection"]').exists()).toBe(false);
  expect(wrapper.find('[data-testid="delete-connection-Packaged"]').exists()).toBe(false);
  expect(wrapper.find('[data-testid="delete-connector"]').exists()).toBe(false);
  expect(wrapper.find('[data-testid="edit-connector"]').exists()).toBe(false);
  expect(wrapper.find('[data-testid="plugin-connector-configuration"]').exists()).toBe(true);
});

it("retains the edit draft and displays a rejected save once", async () => {
  const original = mocks.invoke.getMockImplementation()!;
  mocks.invoke.mockImplementation(async (command, input) => {
    if (command === "plugin:extensions|update_connector_definition") throw new Error("Save rejected");
    return original(command, input);
  });
  const wrapper = await open();
  await wrapper.get('[data-testid="edit-connector"]').trigger("click");
  await wrapper.get('[data-testid="connector-description"]').setValue("Keep this draft");
  await wrapper.get('[data-testid="connector-editor"]').trigger("submit");
  await flushPromises();
  expect(wrapper.findAll('[role="alert"]')).toHaveLength(1);
  expect(wrapper.get('[role="alert"]').text()).toBe("Save rejected");
  expect(wrapper.get('[data-testid="connector-description"]').element).toHaveProperty(
    "value",
    "Keep this draft",
  );
});

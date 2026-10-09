import { QueryClient, VueQueryPlugin } from "@tanstack/vue-query";
import { flushPromises, mount } from "@vue/test-utils";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { i18n } from "../../../foundation/i18n/i18n";
import type { ConnectorDefinition } from "../../../shared/contracts/generated/ConnectorDefinition";
import type { McpConnection } from "../../../shared/contracts/generated/McpConnection";
import { settingsLeaveCoordinatorKey } from "../../../shared/contracts/settings-leave";
import { resolveConfirmation } from "../../../shared/ui/confirm-action";
import ConnectorSettings from "../connectors/ConnectorSettings.vue";
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
      case "plugin:extensions|list_connector_definitions":
        return structuredClone(services);
      case "plugin:extensions|list_mcp_connections":
        return structuredClone(connections);
      case "plugin:extensions|create_mcp_connection": {
        const created = mcpConnectionFixture("new", input!.name as string);
        connections.push(created);
        return created;
      }
      case "plugin:extensions|create_connector_definition": {
        const definition = {
          ...(JSON.parse(JSON.stringify(input!.definition)) as ConnectorDefinition),
          id: "new-service",
          revision: "created",
        };
        services.push(definition);
        return { value: definition, cleanupErrors: [] };
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
  mounted = mount(ConnectorSettings, {
    global: {
      plugins: [i18n, [VueQueryPlugin, { queryClient }]],
      provide: {
        [settingsLeaveCoordinatorKey as symbol]: { register: () => () => {}, requestLeave: async () => {} },
      },
      mocks: { $t: (key: string) => key },
      stubs: { UiDialog: { props: ["open"], template: '<div v-if="open"><slot /></div>' } },
    },
  });
  await flushPromises();
  return mounted;
}

describe("service connector management", () => {
  it("creates another instance from one reusable definition without copying endpoints or auth", async () => {
    const wrapper = await open();
    const connector = wrapper.get('[data-testid="connector-github"]');
    expect(connector.text()).toContain("Read issues and repositories");
    expect(connector.findAll('[data-testid^="mcp-connection-"]')).toHaveLength(0);
    await connector.get('[data-testid="manage-connector-github"]').trigger("click");
    expect(connector.findAll('[data-testid^="mcp-connection-"]')).toHaveLength(2);
    await connector.get('[data-testid="add-connection-instance"]').trigger("click");
    await connector.get('[data-testid="connection-instance-name"]').setValue("Research");
    await connector.get("form").trigger("submit");
    await flushPromises();
    expect(mocks.invoke).toHaveBeenCalledWith("plugin:extensions|create_mcp_connection", {
      connectorId: "github",
      name: "Research",
    });
    expect(services).toHaveLength(1);
    expect(connector.findAll('[data-testid^="mcp-connection-"]')).toHaveLength(3);
  });

  it("creates a service definition through the service editor without an implicit instance", async () => {
    const wrapper = await open();
    await wrapper.get('[data-testid="create-connector"]').trigger("click");
    const editor = wrapper.get('[data-testid="connector-editor"]');
    await editor.get('[data-testid="connector-name"]').setValue("Notion");
    await editor.get('[data-testid="connector-description"]').setValue("Read and update documents");
    await editor.get('[data-testid="mcp-endpoint-url"]').setValue("https://notion.example.com/mcp");
    await editor.trigger("submit");
    await flushPromises();
    expect(mocks.invoke).toHaveBeenCalledWith("plugin:extensions|create_connector_definition", {
      definition: expect.objectContaining({
        name: "Notion",
        description: "Read and update documents",
        endpoint: expect.objectContaining({ url: "https://notion.example.com/mcp" }),
      }),
    });
    expect(connections).toHaveLength(2);
    expect(wrapper.find('[data-testid="connector-editor"]').exists()).toBe(false);
  });

  it("deleting one instance keeps the service and other instances available", async () => {
    const wrapper = await open();
    await wrapper.get('[data-testid="manage-connector-github"]').trigger("click");
    await wrapper.get('[data-testid="delete-connection-Work"]').trigger("click");
    expect(mocks.invoke).not.toHaveBeenCalledWith(
      "plugin:extensions|delete_mcp_connection",
      expect.anything(),
    );
    resolveConfirmation(true);
    await flushPromises();
    expect(mocks.invoke).toHaveBeenCalledWith("plugin:extensions|delete_mcp_connection", {
      connectionId: "Work",
      revision: "instance-one",
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
          ...(JSON.parse(JSON.stringify(input!.definition)) as ConnectorDefinition),
          revision: "updated",
        };
        services = [definition];
        return { value: definition, cleanupErrors: ["credential-cleanup-failed"] };
      }
      return original(command, input);
    });
    const wrapper = await open();
    await wrapper.get('[data-testid="manage-connector-github"]').trigger("click");
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

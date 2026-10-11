import { QueryClient, VueQueryPlugin } from "@tanstack/vue-query";
import { flushPromises, mount } from "@vue/test-utils";
import { afterEach, beforeEach, expect, it, vi } from "vitest";

import { i18n } from "../../../foundation/i18n/instance";
import { settingsLeaveCoordinatorKey } from "../../../shared/contracts/settings-leave";
import McpServerCreation from "../connections/components/McpServerCreation.vue";
import ConnectorRequirements from "../connectors/components/ConnectorRequirements.vue";
import { mcpConnectionFixture } from "./support/connector-fixtures";

const invoke = vi.hoisted(() => vi.fn<(command: string) => Promise<unknown>>());
vi.mock("../../../foundation/ipc/invoke-command", () => ({ invokeCommand: invoke }));
let wrapper: ReturnType<typeof mount>;
let client: QueryClient;
beforeEach(async () => {
  invoke.mockReset();
  invoke.mockImplementation(async (command) => {
    if (command === "plugin:extensions|create_mcp_server")
      return { value: mcpConnectionFixture("created"), cleanupErrors: [] };
    if (
      command === "plugin:extensions|list_connector_definitions" ||
      command === "plugin:extensions|list_mcp_connections"
    )
      return [];
    throw new Error(`Unexpected command: ${command}`);
  });
  client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  wrapper = mount(McpServerCreation, {
    global: {
      plugins: [i18n, [VueQueryPlugin, { queryClient: client }]],
      provide: {
        [settingsLeaveCoordinatorKey as symbol]: { register: () => () => {}, requestLeave: async () => {} },
      },
    },
  });
  await flushPromises();
  await wrapper.get('[data-testid="connector-name"]').setValue("Research");
});
afterEach(() => {
  wrapper.unmount();
  client.clear();
});

it("saves structured STDIO arguments and environment literally with an optional description", async () => {
  wrapper
    .getComponent(ConnectorRequirements)
    .vm.$emit("update:modelValue", { type: "oauth", clientId: "public", scopes: [] });
  await wrapper.get('[data-testid="mcp-transport-stdio"]').trigger("mousedown", { button: 0 });
  await flushPromises();
  expect(wrapper.getComponent(ConnectorRequirements).props("modelValue")).toEqual({ type: "none" });
  await wrapper.get('[data-testid="mcp-executable"]').setValue("/installed/server");
  await wrapper.get('[data-testid="add-mcp-argument"]').trigger("click");
  await wrapper.get('[data-testid="mcp-argument-0"]').setValue("  a path with spaces  ");
  await wrapper.get('[data-testid="add-mcp-argument"]').trigger("click");
  await wrapper.get('[data-testid="add-endpoint-field"]').trigger("click");
  await wrapper.get('[data-testid="endpoint-field-name-0"]').setValue("REGION");
  await wrapper.get('[data-testid="endpoint-field-value-0"]').setValue("west coast");
  expect(wrapper.find("textarea").exists()).toBe(false);
  expect(wrapper.get('[data-testid="save-connector"]').attributes("disabled")).toBeUndefined();
  await wrapper.get('[data-testid="connector-editor"]').trigger("submit");
  await flushPromises();
  expect(invoke).toHaveBeenCalledWith("plugin:extensions|create_mcp_server", {
    definition: expect.objectContaining({
      name: "Research",
      description: "",
      authorization: { type: "none" },
      endpoint: {
        type: "stdio",
        executable: "/installed/server",
        args: ["  a path with spaces  ", ""],
        cwd: null,
        environment: { REGION: "west coast" },
      },
    }),
  });
});

it("rejects case-insensitive duplicate headers without silently overwriting user input", async () => {
  await wrapper.get('[data-testid="mcp-endpoint-url"]').setValue("https://research.example.com/mcp");
  await wrapper.get('[data-testid="add-endpoint-field"]').trigger("click");
  await wrapper.get('[data-testid="add-endpoint-field"]').trigger("click");
  await Promise.all([
    wrapper.get('[data-testid="endpoint-field-name-0"]').setValue("X-Region"),
    wrapper.get('[data-testid="endpoint-field-name-1"]').setValue("x-region"),
    wrapper.get('[data-testid="endpoint-field-value-0"]').setValue("0"),
    wrapper.get('[data-testid="endpoint-field-value-1"]').setValue("1"),
  ]);
  await wrapper.get('[data-testid="connector-editor"]').trigger("submit");
  await flushPromises();
  expect(wrapper.get('[role="alert"]').text()).toBe(
    i18n.global.t("extensions.endpoint.configuration.duplicateName"),
  );
  expect(invoke.mock.calls.some(([command]) => command === "plugin:extensions|create_mcp_server")).toBe(
    false,
  );
  expect(wrapper.get('[data-testid="endpoint-field-value-0"]').element).toHaveProperty("value", "0");
  expect(wrapper.get('[data-testid="endpoint-field-value-1"]').element).toHaveProperty("value", "1");
});

it("replaces transport-specific public fields instead of retaining stale HTTP configuration", async () => {
  await wrapper.get('[data-testid="add-endpoint-field"]').trigger("click");
  await wrapper.get('[data-testid="endpoint-field-name-0"]').setValue("X-Region");
  await wrapper.get('[data-testid="mcp-transport-stdio"]').trigger("mousedown", { button: 0 });
  await flushPromises();
  expect(wrapper.find('[data-testid="endpoint-field-name-0"]').exists()).toBe(false);
  await wrapper.get('[data-testid="mcp-transport-streamable_http"]').trigger("mousedown", { button: 0 });
  await flushPromises();
  expect(wrapper.get('[data-testid="mcp-endpoint-url"]').element).toHaveProperty("value", "");
  expect(wrapper.find('[data-testid="mcp-executable"]').exists()).toBe(false);
});

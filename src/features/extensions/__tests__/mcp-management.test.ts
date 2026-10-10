import { QueryClient, VueQueryPlugin } from "@tanstack/vue-query";
import { flushPromises, mount, type MountingOptions } from "@vue/test-utils";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { i18n } from "../../../foundation/i18n/i18n";
import { ApplicationLeaveCancelled } from "../../../shared/contracts/application-leave";
import { settingsLeaveCoordinatorKey } from "../../../shared/contracts/settings-leave";
import McpConnectionEnablement from "../connections/components/McpConnectionEnablement.vue";
import McpServerCreation from "../connections/components/McpServerCreation.vue";
import { mcpConnectionFixture } from "./support/connector-fixtures";

const mocks = vi.hoisted(() => ({
  invoke: vi.fn<(command: string, input?: Record<string, unknown>) => Promise<unknown>>(),
}));
vi.mock("../../../foundation/ipc/invoke-command", () => ({ invokeCommand: mocks.invoke }));
let wrapper: ReturnType<typeof mount> | undefined;
let queryClient: QueryClient;
const requestLeave = vi.fn<() => Promise<void>>();

beforeEach(() => {
  queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  mocks.invoke.mockReset();
  requestLeave.mockReset().mockResolvedValue();
});
afterEach(() => {
  wrapper?.unmount();
  queryClient.clear();
});
function globalOptions(): NonNullable<MountingOptions<unknown>["global"]> {
  return {
    plugins: [i18n, [VueQueryPlugin, { queryClient }]],
    provide: {
      [settingsLeaveCoordinatorKey as symbol]: { register: () => () => {}, requestLeave },
    },
    stubs: { UiDialog: { props: ["open"], template: '<div v-if="open"><slot /></div>' } },
  };
}

describe("MCP management admission", () => {
  it("creates the service and its first disabled connection through one atomic command", async () => {
    mocks.invoke.mockImplementation(async (command: string) => {
      if (command === "plugin:extensions|create_mcp_server")
        return { value: mcpConnectionFixture("created"), cleanupErrors: [] };
      if (
        command === "plugin:extensions|list_connector_definitions" ||
        command === "plugin:extensions|list_mcp_connections"
      )
        return [];
      throw new Error(`Unexpected command: ${command}`);
    });
    wrapper = mount(McpServerCreation, { global: globalOptions() });
    await flushPromises();
    const editor = wrapper.get('[data-testid="connector-editor"]');
    await editor.get('[data-testid="connector-name"]').setValue("Research");
    await editor.get('[data-testid="connector-description"]').setValue("Read research sources");
    await editor.get('[data-testid="mcp-endpoint-url"]').setValue("https://research.example.com/mcp");
    await editor.trigger("submit");
    await flushPromises();
    expect(mocks.invoke).toHaveBeenCalledWith("plugin:extensions|create_mcp_server", {
      definition: expect.objectContaining({
        name: "Research",
        description: "Read research sources",
        endpoint: expect.objectContaining({ url: "https://research.example.com/mcp", headers: {} }),
      }),
    });
    expect(
      mocks.invoke.mock.calls.some(
        ([command]) => command === "plugin:extensions|create_connector_definition",
      ),
    ).toBe(false);
    expect(
      mocks.invoke.mock.calls.some(([command]) => command === "plugin:extensions|create_mcp_connection"),
    ).toBe(false);
    expect(wrapper.emitted("navigate")).toEqual([[{ tab: "mcp" }]]);
  });

  it("keeps enabled state intact when the user declines to discard unsaved configuration", async () => {
    requestLeave.mockRejectedValue(new ApplicationLeaveCancelled());
    wrapper = mount(McpConnectionEnablement, {
      props: { connection: mcpConnectionFixture("research"), configured: true },
      global: globalOptions(),
    });
    await wrapper.get('[role="switch"]').trigger("click");
    await flushPromises();
    expect(requestLeave).toHaveBeenCalledOnce();
    expect(mocks.invoke).not.toHaveBeenCalled();
    expect(wrapper.get('[role="switch"]').attributes("aria-checked")).toBe("false");
    expect(wrapper.find('[role="alert"]').exists()).toBe(false);
  });

  it("shows backend rejection without presenting the connection as enabled", async () => {
    mocks.invoke.mockRejectedValue(new Error("Connection revision changed"));
    wrapper = mount(McpConnectionEnablement, {
      props: { connection: mcpConnectionFixture("research"), configured: true },
      global: globalOptions(),
    });
    await wrapper.get('[role="switch"]').trigger("click");
    await flushPromises();
    expect(mocks.invoke).toHaveBeenCalledWith("plugin:extensions|update_mcp_connection", {
      connection: expect.objectContaining({ id: "research", enabled: true, revision: "connection-one" }),
    });
    expect(wrapper.get('[role="alert"]').text()).toContain("Connection revision changed");
    expect(wrapper.get('[role="switch"]').attributes("aria-checked")).toBe("false");
  });
});

import { QueryClient, VueQueryPlugin } from "@tanstack/vue-query";
import { flushPromises, mount } from "@vue/test-utils";
import { afterEach, beforeEach, expect, it, vi } from "vitest";

import { i18n } from "../../../foundation/i18n/instance";
import type { McpConnection } from "../../../shared/contracts/generated/McpConnection";
import type { SettingsLeaveParticipant } from "../../../shared/contracts/settings-leave";
import { settingsLeaveCoordinatorKey } from "../../../shared/contracts/settings-leave";
import McpConnectionSettings from "../connections/components/McpConnectionSettings.vue";
import McpToolSelection from "../connections/components/McpToolSelection.vue";
import { connectorFixture, mcpConnectionFixture } from "./support/connector-fixtures";
const invoke = vi.hoisted(() =>
  vi.fn<(command: string, input?: Record<string, unknown>) => Promise<unknown>>(),
);
vi.mock("../../../foundation/ipc/invoke-command", () => ({ invokeCommand: invoke }));
let wrapper: ReturnType<typeof mount>;
let queryClient: QueryClient;
let participant: SettingsLeaveParticipant;
const connection = {
  ...mcpConnectionFixture("work"),
  tools: [{ name: "search", schemaDigest: "one", readOnly: true, policy: "allow" as const }],
};
beforeEach(async () => {
  invoke.mockReset();
  invoke.mockImplementation(async (command: string, input?: Record<string, unknown>) => {
    if (command === "plugin:extensions|check_mcp_connection")
      return {
        tools: [{ name: "search", description: "Search", schemaDigest: "one", readOnlyHint: true }],
        resources: [],
        prompts: [],
        resourceTemplates: [],
      };
    if (command === "plugin:extensions|update_mcp_connection")
      return { ...(input!.connection as McpConnection), revision: "saved" };
    throw new Error(`Unexpected command: ${command}`);
  });
  queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  wrapper = mount(McpConnectionSettings, {
    props: { connection, connector: connectorFixture },
    global: {
      plugins: [i18n, [VueQueryPlugin, { queryClient }]],
      mocks: { $t: (key: string) => key },
      provide: {
        [settingsLeaveCoordinatorKey as symbol]: {
          register: (registered: SettingsLeaveParticipant) => {
            participant = registered;
            return () => {};
          },
          requestLeave: async () => {},
        },
      },
    },
  });
  await flushPromises();
});
afterEach(() => {
  wrapper.unmount();
  queryClient.clear();
});

it("saves the name and reviewed tool draft together, and does not reset them on catalog inspection", async () => {
  await wrapper.get('[data-testid="check-mcp-connection"]').trigger("click");
  await flushPromises();
  expect(wrapper.getComponent(McpToolSelection).props("modelValue")).toEqual(connection.tools);
  await wrapper.get("input").setValue("Renamed");
  wrapper
    .getComponent(McpToolSelection)
    .vm.$emit("update:modelValue", [{ ...connection.tools[0]!, policy: "confirm" }]);
  await wrapper.get("form").trigger("submit");
  await flushPromises();
  expect(invoke).toHaveBeenCalledWith("plugin:extensions|update_mcp_connection", {
    connection: expect.objectContaining({
      name: "Renamed",
      tools: [expect.objectContaining({ policy: "confirm" })],
    }),
  });
  await expect(participant.prepare()).resolves.toBe(true);
});

it("retains unsaved edits when a newer revision arrives, and refuses a stale save", async () => {
  await wrapper.get("input").setValue("Unsaved");
  await wrapper.setProps({ connection: { ...connection, name: "Elsewhere", revision: "newer" } });
  expect(wrapper.get("input").element).toHaveProperty("value", "Unsaved");
  expect(wrapper.text()).toContain("extensions.connections.conflict");
  expect(wrapper.get('button[type="submit"]').attributes("disabled")).toBeDefined();
  participant.finalize?.();
  await flushPromises();
  expect(wrapper.get("input").element).toHaveProperty("value", "Elsewhere");
});

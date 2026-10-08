import { QueryClient, VueQueryPlugin } from "@tanstack/vue-query";
import { flushPromises, mount } from "@vue/test-utils";
import { expect, it, vi } from "vitest";

import type { ConnectorAccount } from "../../../shared/contracts/generated/ConnectorAccount";
import ConnectorAccounts from "../connectors/ConnectorAccounts.vue";

const mocks = vi.hoisted(() => ({
  invoke: vi.fn<(command: string, payload?: Record<string, unknown>) => Promise<unknown>>(),
}));
vi.mock("../../../foundation/ipc/invoke-command", () => ({ invokeCommand: mocks.invoke }));

it("refreshes authorization state after a failed authorization flow", async () => {
  let account: ConnectorAccount = {
    id: "account",
    revision: "one",
    connectorId: "service",
    displayName: "Work",
    resource: "https://example.com/mcp",
    issuer: null,
    scopes: [],
    authorization: { type: "bearer" },
    enabled: true,
    cleanupPending: false,
  };
  mocks.invoke.mockImplementation(async (command: string) => {
    if (command === "plugin:extensions|list_connector_accounts") return [{ ...account }];
    if (command === "plugin:extensions|authorize_connector_account") {
      account = { ...account, enabled: false, cleanupPending: true };
      throw new Error("credential-write-failed");
    }
    throw new Error(`Unexpected command: ${command}`);
  });
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  const wrapper = mount(ConnectorAccounts, {
    props: {
      connector: {
        id: "service",
        revision: "one",
        name: "Service",
        description: "Reference",
        pluginId: null,
        componentName: null,
        authorization: account.authorization,
        endpoint: { type: "streamable_http", url: account.resource, headers: {}, allowLocalNetwork: false },
      },
    },
    global: { plugins: [[VueQueryPlugin, { queryClient }]], mocks: { $t: (key: string) => key } },
  });
  try {
    await flushPromises();
    const form = wrapper.get('[data-testid="connector-authorization"]');
    expect(form.text()).toContain("extensions.accounts.authorized");
    await form.get('input[type="password"]').setValue("submitted-token");
    await form.trigger("submit");
    await flushPromises();
    expect(form.text()).toContain("extensions.accounts.cleanupPending");
    expect(form.text()).toContain("credential-write-failed");
    expect(form.get('input[type="password"]').element).toHaveProperty("value", "");
  } finally {
    wrapper.unmount();
    queryClient.clear();
  }
});

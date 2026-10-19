import { QueryClient, VueQueryPlugin } from "@tanstack/vue-query";
import { flushPromises, mount } from "@vue/test-utils";
import { afterEach, expect, it, vi } from "vitest";

import { i18n } from "../../../foundation/i18n/instance";
import { settingsLeaveCoordinatorKey } from "../../../shared/contracts/settings-leave";
import AppInspection from "../apps/components/AppInspection.vue";
import AppSettings from "../apps/components/AppSettings.vue";
import McpSettings from "../connections/components/McpSettings.vue";

const invoke = vi.hoisted(() => vi.fn<(command: string) => Promise<unknown>>());
vi.mock("../../../foundation/ipc/invoke-command", () => ({ invokeCommand: invoke }));
const app = {
  id: "github",
  revision: "one",
  name: "GitHub",
  description: "Repository knowledge",
  pluginId: "github-work",
  componentName: "github",
  endpoint: {
    type: "streamable_http",
    url: "https://api.githubcopilot.com/mcp/",
    headers: {},
    allowLocalNetwork: false,
  },
  authorization: { type: "bearer" },
};
const connection = {
  id: "packaged",
  revision: "one",
  name: "GitHub",
  connectorId: "github",
  enabled: true,
  tools: [],
};
const plugin = {
  id: "github-work",
  enabled: true,
  source: { type: "bundled", name: "github-work" },
  revision: {
    components: [{ name: "github", kind: "app", supported: true, diagnostic: null }],
    manifest: { name: "github-work" },
    connectors: [
      { componentName: "github", configuration: { type: "streamable-http", url: app.endpoint.url } },
    ],
  },
};
let wrapper: ReturnType<typeof mount>;
let client: QueryClient;
afterEach(() => {
  wrapper.unmount();
  client.clear();
});
async function render(connectorId?: string, tab = "apps") {
  client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  invoke.mockImplementation(async (command) => {
    switch (command) {
      case "plugin:extensions|list_connector_definitions":
        return [
          app,
          { ...app, id: "custom", pluginId: null },
          { ...app, id: "protocol", componentName: "raw-mcp" },
        ];
      case "plugin:extensions|list_mcp_connections":
        return [connection, { ...connection, id: "protocol", connectorId: "protocol", enabled: false }];
      case "plugin:extensions|list_plugin_installations":
        return [plugin];
      case "plugin:extensions|list_connector_accounts":
        return [];
      case "plugin:extensions|get_agent_extension_availability":
        return {
          plugins: [plugin],
          connections: [],
          skills: [],
          connectors: [app],
          diagnostics: [],
          defaultSelection: { pluginIds: [], skillIds: [], connections: [] },
        };
      default:
        throw new Error(`Unexpected command: ${command}`);
    }
  });
  wrapper = mount(tab === "apps" ? AppSettings : McpSettings, {
    props: { search: "", ...(connectorId ? { connectorId } : {}) },
    global: {
      plugins: [i18n, [VueQueryPlugin, { queryClient: client }]],
      provide: { [settingsLeaveCoordinatorKey as symbol]: { register: () => () => {} } },
      stubs: { UiSettingsHeaderActions: { template: "<div><slot /></div>" } },
    },
  });
  await flushPromises();
}

it("lists packaged integrations separately from custom MCP and navigates to the catalog", async () => {
  await render();
  expect(wrapper.find('[data-testid="app-github"]').exists()).toBe(true);
  expect(wrapper.find('[data-testid="app-custom"]').exists()).toBe(false);
  expect(wrapper.find('[data-testid="app-protocol"]').exists()).toBe(false);
  expect(wrapper.find('[role="switch"]').exists()).toBe(false);
  await wrapper.get('[data-testid="browse-app-plugins"]').trigger("click");
  expect(wrapper.emitted("navigate")?.at(-1)).toEqual([{ tab: "plugins", action: "browse" }]);
  await wrapper.get('[data-testid="app-github"] button').trigger("click");
  expect(wrapper.emitted("navigate")?.at(-1)).toEqual([{ tab: "apps", connectorId: "github" }]);
});

it("connects a packaged app in one action, without a custom endpoint form or component switch", async () => {
  await render("github");
  expect(wrapper.findComponent(AppInspection).exists()).toBe(true);
  expect(wrapper.find('[data-testid="connect-connector"]').exists()).toBe(true);
  expect(wrapper.find('[data-testid="select-connector-account"]').exists()).toBe(false);
  expect(wrapper.find('[data-testid="check-mcp-connection"]').exists()).toBe(true);
  expect(wrapper.find('[data-testid="plugin-connector-configuration"]').exists()).toBe(false);
  expect(wrapper.find('[role="switch"]').exists()).toBe(false);
  expect(wrapper.find('[data-testid="add-mcp-server"]').exists()).toBe(false);
  await wrapper.get('[data-testid="extension-back"]').trigger("click");
  expect(wrapper.emitted("navigate")?.at(-1)).toEqual([{ tab: "apps" }]);
});

it("keeps packaged raw services in MCP with enablement owned by the plugin", async () => {
  await render(undefined, "mcp");
  expect(wrapper.find('[data-testid="mcp-server-packaged"]').exists()).toBe(false);
  expect(wrapper.find('[data-testid="mcp-server-protocol"]').exists()).toBe(true);
  expect(wrapper.find('[role="switch"]').exists()).toBe(false);
  await wrapper.get('[data-testid="mcp-server-protocol"] button').trigger("click");
  expect(wrapper.emitted("navigate")?.at(-1)).toEqual([{ tab: "mcp", id: "protocol" }]);
});

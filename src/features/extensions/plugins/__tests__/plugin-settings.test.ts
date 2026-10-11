import { QueryClient, VueQueryPlugin } from "@tanstack/vue-query";
import { flushPromises, mount } from "@vue/test-utils";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import type * as PluginOperations from "../../../../foundation/extensions/plugin-operations";
import { i18n } from "../../../../foundation/i18n/instance";
import type { PluginInspection } from "../../../../shared/contracts/generated/PluginInspection";
import type { PluginInstallation } from "../../../../shared/contracts/generated/PluginInstallation";
import PluginSettings from "../components/PluginSettings.vue";

const mocks = vi.hoisted(() => ({
  list: vi.fn<typeof PluginOperations.listPluginInstallations>(),
  inspectSource: vi.fn<typeof PluginOperations.inspectPluginSource>(),
  inspectUpdate: vi.fn<typeof PluginOperations.inspectPluginUpdate>(),
  install: vi.fn<typeof PluginOperations.installPlugin>(),
  update: vi.fn<typeof PluginOperations.updatePlugin>(),
}));
vi.mock("../../../../foundation/extensions/plugin-operations", () => ({
  listPluginInstallations: mocks.list,
  inspectPluginSource: mocks.inspectSource,
  inspectPluginUpdate: mocks.inspectUpdate,
  installPlugin: mocks.install,
  updatePlugin: mocks.update,
  enablePlugin: vi.fn<typeof PluginOperations.enablePlugin>(),
  disablePlugin: vi.fn<typeof PluginOperations.disablePlugin>(),
  uninstallPlugin: vi.fn<typeof PluginOperations.uninstallPlugin>(),
  searchPlugins: vi.fn<typeof PluginOperations.searchPlugins>().mockResolvedValue([]),
  addPluginMarketplace: vi.fn<typeof PluginOperations.addPluginMarketplace>(),
  refreshPluginMarketplace: vi.fn<typeof PluginOperations.refreshPluginMarketplace>(),
  removePluginMarketplace: vi.fn<typeof PluginOperations.removePluginMarketplace>(),
}));

const inspection: PluginInspection = {
  manifest: {
    $schema: "https://knovum.test/plugin.schema.json",
    name: "Example plugin",
    version: "1.0.0",
    description: "Example description",
    author: null,
    homepage: null,
    repository: null,
    license: null,
    keywords: null,
    extensions: null,
  },
  digest: "digest",
  sourceRevision: "revision",
  components: [{ name: "example", kind: "skill", supported: true, diagnostic: null }],
  connectors: [],
  fileCount: 1,
  byteLength: 10,
};
const installed: PluginInstallation = {
  id: "example",
  source: { type: "directory", path: "/installed" },
  revision: inspection,
  enabled: true,
  builtin: false,
  publication: null,
};

let queryClient: QueryClient;
let wrapper: ReturnType<typeof mount>;

beforeEach(async () => {
  vi.clearAllMocks();
  queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  mocks.list.mockResolvedValue([installed]);
  mocks.inspectSource.mockResolvedValue(inspection);
  mocks.inspectUpdate.mockResolvedValue({ source: installed.source, inspection });
  mocks.install.mockResolvedValue({ value: installed, cleanupErrors: [] });
  mocks.update.mockResolvedValue({ value: installed, cleanupErrors: [] });
  wrapper = mount(PluginSettings, {
    global: {
      plugins: [i18n, [VueQueryPlugin, { queryClient }]],
      mocks: { $t: (key: string) => key },
      stubs: {
        UiSettingsHeaderActions: { template: "<div><slot /></div>" },
        UiDialog: { props: ["open"], template: '<div v-if="open"><slot /></div>' },
        PluginDiscovery: {
          template:
            "<button data-testid=\"inspect-release\" @click=\"$emit('inspect', { type: 'directory', path: '/catalog' }, { marketplaceId: 'market', publisherId: 'publisher', pluginName: 'Example plugin' })\">inspect</button>",
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

async function inspectLocalSource() {
  await wrapper.get('[data-testid="open-plugin-source"]').trigger("click");
  await wrapper.get('[data-testid="plugin-source-location"]').setValue("/local");
  await wrapper.get("form").trigger("submit");
  await flushPromises();
}

describe("plugin installation review", () => {
  it("keeps the source dialog open and shows the inspection error when inspection fails", async () => {
    mocks.inspectSource.mockRejectedValueOnce(new Error("inspection failed"));

    await inspectLocalSource();

    expect(wrapper.find('[data-testid="plugin-source-location"]').exists()).toBe(true);
    expect(wrapper.text()).toContain("inspection failed");
  });

  it("closes the source dialog only after inspection succeeds", async () => {
    await inspectLocalSource();

    expect(wrapper.find('[data-testid="plugin-source-location"]').exists()).toBe(false);
    expect(wrapper.find('[data-testid="plugin-installation-review"]').exists()).toBe(true);
  });

  it("returns to the installed view after a successful catalog installation", async () => {
    await wrapper.get('[data-testid="discover-plugins"]').trigger("click");
    expect(wrapper.emitted("navigate")?.at(-1)).toEqual([{ tab: "plugins", action: "browse" }]);
    await wrapper.setProps({ action: "browse" });
    await wrapper.get('[data-testid="inspect-release"]').trigger("click");
    await flushPromises();
    await wrapper.get('[data-testid="confirm-plugin-installation"]').trigger("click");
    await flushPromises();

    expect(wrapper.emitted("navigate")?.at(-1)).toEqual([{ tab: "plugins" }]);
    await wrapper.setProps({ action: undefined });
    expect(wrapper.find('[data-testid="inspect-release"]').exists()).toBe(false);
    expect(wrapper.find('[data-testid="plugin-example"]').exists()).toBe(true);
    expect(mocks.list).toHaveBeenCalledTimes(2);
  });

  it("keeps the installation review open and shows the mutation error when installation fails", async () => {
    mocks.install.mockRejectedValueOnce(new Error("installation failed"));
    await inspectLocalSource();

    await wrapper.get('[data-testid="confirm-plugin-installation"]').trigger("click");
    await flushPromises();

    expect(wrapper.find('[data-testid="plugin-installation-review"]').exists()).toBe(true);
    expect(wrapper.text()).toContain("installation failed");
  });
});

import { QueryClient, VueQueryPlugin } from "@tanstack/vue-query";
import { flushPromises, mount } from "@vue/test-utils";
import { afterEach, expect, it, vi } from "vitest";

import { i18n } from "../../../../foundation/i18n/instance";
import PluginDiscovery from "../components/PluginDiscovery.vue";

const invoke = vi.hoisted(() => vi.fn<(command: string) => Promise<unknown>>());
vi.mock("../../../../foundation/ipc/invoke-command", () => ({ invokeCommand: invoke }));
const source = { type: "bundled", name: "study-companion" };
const releases = [
  { publisherId: "knovum", name: "study-companion", description: "Study from evidence", source },
  {
    publisherId: "knovum",
    name: "meeting-notes",
    description: "Meeting notes",
    source: { type: "bundled", name: "meeting-notes" },
  },
];
let wrapper: ReturnType<typeof mount>;
let client: QueryClient;
afterEach(() => {
  wrapper.unmount();
  client.clear();
});

it("offers bundled installation with publication identity and routes installed releases to inspection", async () => {
  client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  invoke.mockImplementation(async (command) => {
    if (command === "plugin:extensions|search_plugins")
      return [
        {
          id: "knovum-bundled",
          name: "knovum",
          displayName: "Knovum",
          source: { type: "bundled" },
          releases,
        },
      ];
    if (command === "plugin:extensions|list_plugin_installations")
      return [{ id: "study", source, publication: null }];
    throw new Error(`Unexpected command: ${command}`);
  });
  wrapper = mount(PluginDiscovery, {
    props: { busy: false, search: "" },
    global: { plugins: [i18n, [VueQueryPlugin, { queryClient: client }]] },
  });
  await flushPromises();
  expect(wrapper.find('[data-testid="preview-plugin-study-companion"]').exists()).toBe(false);
  await wrapper.get('[data-testid="installed-plugin-study-companion"]').trigger("click");
  expect(wrapper.emitted("navigate")).toEqual([["study"]]);
  await wrapper.get('[data-testid="preview-plugin-meeting-notes"]').trigger("click");
  expect(wrapper.emitted("inspect")).toEqual([
    [
      { type: "bundled", name: "meeting-notes" },
      { marketplaceId: "knovum-bundled", publisherId: "knovum", pluginName: "meeting-notes" },
    ],
  ]);
  await wrapper.setProps({ search: "missing" });
  expect(wrapper.find('[data-testid="preview-plugin-meeting-notes"]').exists()).toBe(false);
});

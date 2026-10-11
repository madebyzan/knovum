import { QueryClient, VueQueryPlugin } from "@tanstack/vue-query";
import { flushPromises, mount } from "@vue/test-utils";
import { afterEach, expect, it, vi } from "vitest";

import { i18n } from "../../../foundation/i18n/instance";
import type { PluginInstallation } from "../../../shared/contracts/generated/PluginInstallation";
import PluginContents from "../plugins/components/PluginContents.vue";
import SkillActions from "../skills/components/SkillActions.vue";

const invoke = vi.hoisted(() => vi.fn<(command: string) => Promise<unknown>>());
vi.mock("../../../foundation/ipc/invoke-command", () => ({ invokeCommand: invoke }));
const plugin: PluginInstallation = {
  id: "owner",
  enabled: true,
  builtin: true,
  publication: null,
  source: { type: "directory", path: "/bundle" },
  revision: {
    digest: "package",
    sourceRevision: "source",
    components: [],
    connectors: [],
    fileCount: 1,
    byteLength: 1,
    manifest: {
      $schema: "https://test/plugin.schema.json",
      name: "Knowledge",
      version: "1",
      description: "Knowledge work",
      author: null,
      homepage: null,
      repository: null,
      license: null,
      keywords: null,
      extensions: null,
    },
  },
};
const skill = {
  id: "packaged",
  pluginId: "owner",
  name: "evidence-brief",
  description: "Compare sources",
  digest: "one",
  enabled: true,
  requirements: [],
  includedTools: [],
  unsupportedRequirements: [],
};
const client = new QueryClient({ defaultOptions: { queries: { retry: false } } });
let wrapper: ReturnType<typeof mount>;
afterEach(() => {
  wrapper.unmount();
  client.clear();
});

it("inspects packaged instructions within the plugin with no child switch and can copy to personal ownership", async () => {
  invoke.mockImplementation(async (command) => {
    switch (command) {
      case "plugin:extensions|list_skills":
        return [skill];
      case "plugin:extensions|list_connector_definitions":
      case "plugin:extensions|list_mcp_connections":
        return [];
      case "plugin:extensions|list_plugin_installations":
        return [plugin];
      case "plugin:extensions|get_agent_extension_availability":
        return {
          plugins: [],
          skills: [skill],
          connections: [],
          connectors: [],
          diagnostics: [],
          defaultSelection: { pluginIds: [], skillIds: [], connections: [] },
        };
      case "plugin:extensions|read_skill":
        return { skill, markdown: "# Compare sources\nUse evidence.", files: [] };
      case "plugin:extensions|copy_skill":
        return { value: { ...skill, id: "personal", pluginId: null }, cleanupErrors: [] };
      default:
        throw new Error(`Unexpected command: ${command}`);
    }
  });
  wrapper = mount(PluginContents, {
    props: { plugin, busy: false },
    global: { plugins: [i18n, [VueQueryPlugin, { queryClient: client }]] },
  });
  await flushPromises();
  expect(wrapper.findAll('[role="switch"]')).toHaveLength(1);
  await wrapper.get('[data-testid="plugin-skill-packaged"] button').trigger("click");
  await flushPromises();
  expect(wrapper.findAll('[role="switch"]')).toHaveLength(0);
  expect(wrapper.find(".ui-markdown").text()).toContain("Use evidence.");
  expect(wrapper.emitted("navigate")).toBeUndefined();
  wrapper.getComponent(SkillActions).vm.$emit("copy");
  await flushPromises();
  expect(invoke).toHaveBeenCalledWith("plugin:extensions|copy_skill", { skillId: "packaged" });
  await wrapper.get('[data-testid="extension-back"]').trigger("click");
  expect(wrapper.find('[data-testid="plugin-skill-packaged"]').exists()).toBe(true);
  expect(wrapper.emitted("navigate")).toBeUndefined();
});

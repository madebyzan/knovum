import { QueryClient, VueQueryPlugin } from "@tanstack/vue-query";
import { flushPromises, mount } from "@vue/test-utils";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { defineComponent } from "vue";
import { createMemoryHistory, createRouter, RouterView } from "vue-router";

import { skillQuery } from "../../../foundation/extensions/extension-query";
import { i18n } from "../../../foundation/i18n/instance";
import { ApplicationLeaveCancelled } from "../../../shared/contracts/application-leave";
import { settingsLeaveCoordinatorKey } from "../../../shared/contracts/settings-leave";
import AgentExtensionsSettings from "../AgentExtensionsSettings.vue";

const invoke = vi.hoisted(() =>
  vi.fn<(command: string, input?: Record<string, unknown>) => Promise<unknown>>(),
);
vi.mock("../../../foundation/ipc/invoke-command", () => ({ invokeCommand: invoke }));
const skill = {
  id: "research",
  name: "evidence-brief",
  description: "Compare evidence",
  pluginId: null,
  digest: "one",
  enabled: true,
  requirements: [],
  includedTools: [],
  unsupportedRequirements: [],
};
let wrapper: ReturnType<typeof mount>;
let queryClient: QueryClient;
let canLeave = true;
const router = createRouter({
  history: createMemoryHistory(),
  routes: [{ path: "/settings", component: AgentExtensionsSettings }],
});
beforeEach(async () => {
  canLeave = true;
  invoke.mockReset();
  invoke.mockImplementation(async (command: string) => {
    switch (command) {
      case "plugin:extensions|list_skills":
        return [skill];
      case "plugin:extensions|read_skill":
        return {
          skill,
          markdown:
            "---\nname: evidence-brief\ndescription: Compare evidence\n---\n\n# Instructions\nRead the sources.",
          files: [],
        };
      case "plugin:extensions|list_plugin_installations":
      case "plugin:extensions|list_connector_definitions":
      case "plugin:extensions|list_mcp_connections":
        return [];
      case "plugin:extensions|get_agent_extension_availability":
        return {
          plugins: [],
          skills: [skill],
          connections: [],
          connectors: [],
          diagnostics: [],
          defaultSelection: { pluginIds: [], skillIds: [], connections: [] },
        };
      default:
        throw new Error(`Unexpected command: ${command}`);
    }
  });
  queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  await router.replace("/settings?pane=extensions");
  wrapper = mount(
    defineComponent({
      components: { RouterView },
      template:
        '<div><header><h1>Extensions</h1><div id="settings-header-actions" /></header><RouterView /></div>',
    }),
    {
      attachTo: document.body,
      global: {
        plugins: [router, i18n, [VueQueryPlugin, { queryClient }]],
        provide: {
          [settingsLeaveCoordinatorKey as symbol]: {
            register: () => () => {},
            requestLeave: async () => {
              if (!canLeave) throw new ApplicationLeaveCancelled();
            },
          },
        },
        mocks: { $t: (key: string) => key },
      },
    },
  );
  await flushPromises();
});
afterEach(() => {
  wrapper.unmount();
  queryClient.clear();
});

it("counts and searches all four categories, and opens readable skill content", async () => {
  expect(wrapper.findAll('[data-testid^="extensions-tab-"]')).toHaveLength(4);
  expect(wrapper.get('[data-testid="extensions-tab-skills"]').text()).toContain("1");
  await wrapper.get('[data-testid="extensions-tab-skills"]').trigger("mousedown", { button: 0 });
  await flushPromises();
  const search = wrapper.get('[data-testid="extension-search"] input');
  await search.setValue("missing");
  expect(wrapper.find('[data-testid="skill-research"]').exists()).toBe(false);
  await search.setValue("evidence");
  await wrapper.get('[data-testid="skill-research"] button').trigger("click");
  await flushPromises();
  expect(router.currentRoute.value.query.extensionId).toBe("research");
  expect(wrapper.find('[data-testid="extension-search"]').exists()).toBe(false);
  expect(wrapper.find("textarea").exists()).toBe(false);
  expect(wrapper.find(".ui-markdown").text()).toContain("Read the sources.");
  expect(wrapper.find(".ui-markdown").text()).not.toContain("description:");
});

it("guards tab changes and history navigation without losing a cancelled draft", async () => {
  canLeave = false;
  await wrapper.get('[data-testid="extensions-tab-skills"]').trigger("mousedown", { button: 0 });
  await flushPromises();
  expect(router.currentRoute.value.query.extensionTab).toBeUndefined();
  canLeave = true;
  await router.push("/settings?pane=extensions&extensionTab=skills");
  await router.push("/settings?pane=extensions&extensionTab=skills&extensionId=research");
  await flushPromises();
  canLeave = false;
  router.back();
  await flushPromises();
  expect(router.currentRoute.value.query.extensionId).toBe("research");
  canLeave = true;
  await wrapper.get('[data-testid="extension-back"]').trigger("click");
  await flushPromises();
  expect(router.currentRoute.value.query.extensionId).toBeUndefined();
});

it("counts and lists only personal Skills, and rejects packaged Skills in the personal inspection", async () => {
  queryClient.setQueryData(skillQuery.queryKey, [skill, { ...skill, id: "packaged", pluginId: "owner" }]);
  await flushPromises();
  expect(wrapper.get('[data-testid="extensions-tab-skills"]').text()).toContain("1");
  await router.push("/settings?pane=extensions&extensionTab=skills");
  await flushPromises();
  expect(wrapper.find('[data-testid="skill-research"]').exists()).toBe(true);
  expect(wrapper.find('[data-testid="skill-packaged"]').exists()).toBe(false);
  await router.push("/settings?pane=extensions&extensionTab=skills&extensionId=packaged");
  await flushPromises();
  expect(wrapper.find(".ui-markdown").exists()).toBe(false);
  expect(wrapper.text()).toContain("extensions.overview.notFound");
});

it("places category actions in the settings title area and hides them during inspection", async () => {
  expect(wrapper.find('header [data-testid="open-plugin-source"]').exists()).toBe(true);
  expect(wrapper.find('header [data-testid="discover-plugins"]').exists()).toBe(true);
  expect(
    wrapper
      .get('[data-testid="agent-extensions-settings"]')
      .find('[data-testid="open-plugin-source"]')
      .exists(),
  ).toBe(false);
  await wrapper.get('[data-testid="extensions-tab-skills"]').trigger("mousedown", { button: 0 });
  await flushPromises();
  expect(wrapper.find('header [data-testid="create-skill"]').exists()).toBe(true);
  expect(wrapper.find('header [data-testid="open-plugin-source"]').exists()).toBe(false);
  await wrapper.get('[data-testid="skill-research"] button').trigger("click");
  await flushPromises();
  expect(wrapper.get("#settings-header-actions").text()).toBe("");
});
